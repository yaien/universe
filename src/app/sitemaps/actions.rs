use chrono::Utc;
use sqlx::prelude::FromRow;

use crate::app::{AppError, AppResult};
use crate::infra::{DbPool, Id};

#[derive(FromRow)]
pub struct Action {
    pub id: Id,
    pub name: String,
    pub tool: String,
    pub codename: String,
    pub response_body_template: String,
}

pub struct Actions {
    pool: DbPool,
}

impl Actions {
    pub async fn get_by_sitemap_id(&self, sitemap_id: &Id) -> AppResult<Vec<Action>> {
        sqlx::query_as::<_, Action>("select * from action where sitemap_id = $1")
            .bind(sitemap_id)
            .fetch_all(&self.pool)
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
        sqlx::query("update actions set name = $1, codename = $2, tool = $3, response_body_template = $4, updated_at = $5) where sitemap_id = $7 and id = &7")
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
}

pub struct CreateActionOptions<'a> {
    sitemap_id: &'a Id,
    name: &'a str,
    codename: &'a str,
    tool: &'a str,
}

pub struct UpdateActionOptions<'a> {
    sitemap_id: &'a Id,
    action_id: &'a Id,
    name: &'a str,
    codename: &'a str,
    tool: &'a str,
    response_body_template: &'a str,
}
