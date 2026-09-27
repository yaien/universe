use std::sync::Arc;

use actix_session::Session;
use actix_web::HttpResponse;
use anyhow::Context;
use chrono::Utc;
use minijinja::{Environment, context};
use sqlx::prelude::FromRow;

use crate::app::auth::{Organization, User};
use crate::app::sitemaps::contents::{RegisterFunctions, RegistryContext, Tool, ToolContext};
use crate::app::{App, AppError, AppResult};
use crate::infra::{DbPool, Id};

#[derive(FromRow)]
pub struct Action {
    pub id: Id,
    pub name: String,
    pub tool: Tool,
    pub codename: String,
    pub response_body_template: String,
}

pub struct Actions {
    pool: DbPool,
}

impl Actions {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_sitemap_id(&self, sitemap_id: &Id) -> AppResult<Vec<Action>> {
        sqlx::query_as::<_, Action>("select * from actions where sitemap_id = $1")
            .bind(sitemap_id)
            .fetch_all(&self.pool)
            .await
            .map_err(AppError::Sqlx)
    }

    pub async fn get_one_by_sitemap_id(
        &self,
        sitemap_id: &Id,
        action_id: &Id,
    ) -> AppResult<Action> {
        sqlx::query_as::<_, Action>("select * from actions where sitemap_id = $1 and id = $2")
            .bind(sitemap_id)
            .bind(action_id)
            .fetch_one(&self.pool)
            .await
            .map_err(AppError::Sqlx)
    }

    pub async fn get_one_by_sitemap_id_and_codename(
        &self,
        sitemap_id: &Id,
        codename: &str,
    ) -> AppResult<Action> {
        sqlx::query_as::<_, Action>("select * from actions where sitemap_id = $1 and codename = $2")
            .bind(sitemap_id)
            .bind(codename)
            .fetch_one(&self.pool)
            .await
            .map_err(AppError::Sqlx)
    }

    pub async fn create<'a>(&self, opts: CreateActionOptions<'a>) -> AppResult<Action> {
        sqlx::query_as::<_, Action>("insert into actions (sitemap_id, name, codename, tool) values ($1, $2, $3, $4) returning *")
            .bind(opts.sitemap_id)
            .bind(opts.name)
            .bind(opts.codename)
            .bind(opts.tool).fetch_one(&self.pool)
            .await
            .map_err(AppError::Sqlx)
    }

    pub async fn update<'a>(&self, opts: UpdateActionOptions<'a>) -> AppResult<()> {
        sqlx::query("update actions set name = $1, codename = $2, tool = $3, response_body_template = $4, updated_at = $5 where sitemap_id = $6 and id = $7")
            .bind(opts.name)
            .bind(opts.codename)
            .bind(opts.tool)
            .bind(opts.response_body_template)
            .bind(Utc::now())
            .bind(opts.sitemap_id)
            .bind(opts.action_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn delete(&self, sitemap_id: &Id, action_id: &Id) -> AppResult<()> {
        sqlx::query("delete from actions where sitemap_id = $1 and id = $2")
            .bind(sitemap_id)
            .bind(action_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

pub struct CreateActionOptions<'a> {
    pub sitemap_id: &'a Id,
    pub name: &'a str,
    pub codename: &'a str,
    pub tool: &'a Tool,
}

pub struct UpdateActionOptions<'a> {
    pub sitemap_id: &'a Id,
    pub action_id: &'a Id,
    pub name: &'a str,
    pub codename: &'a str,
    pub tool: &'a Tool,
    pub response_body_template: &'a str,
}

pub struct ActionContext {
    pub app: Arc<App>,
    pub org: Arc<Organization>,
    pub user: Arc<Option<User>>,
    pub session: Arc<Session>,
    pub inline: bool,
}

impl Action {
    pub async fn call<'a>(&self, ctx: ActionContext) -> AppResult<HttpResponse> {
        let mut env = Environment::new();

        let registry_context = RegistryContext {
            app: ctx.app.clone(),
            org: ctx.org.clone(),
            user: ctx.user.clone(),
        };

        env.register_functions(&registry_context, ctx.inline);

        let tool_context = ToolContext {
            app: ctx.app.clone(),
            org: ctx.org.clone(),
            user: ctx.user.clone(),
            session: ctx.session.clone(),
            inline: ctx.inline,
        };

        let output = self
            .tool
            .call(tool_context)
            .context("failed calling tool")?;
        let body = env
            .render_str(
                &self.response_body_template,
                context! {
                    user => ctx.user,
                    org => ctx.org,
                    output => output,
                },
            )
            .context("failed at render response template body")?;

        Ok(HttpResponse::Ok().body(body))
    }
}
