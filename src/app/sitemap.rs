use std::sync::Arc;

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;

use crate::app::{Colors, Emails, Fonts, Layouts, Pages};
use crate::infra::{DbPool, Id};

pub struct Branch;

impl Branch {
    pub const MAIN: &'static str = "main";
    pub const DRAFT: &'static str = "draft";
}

#[derive(FromRow)]
pub struct Sitemap {
    pub id: Id,
    pub organization_id: Id,
    pub branch: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct Sitemaps {
    pool: DbPool,
    pages: Arc<Pages>,
    emails: Arc<Emails>,
    fonts: Arc<Fonts>,
    colors: Arc<Colors>,
    layouts: Arc<Layouts>,
}

impl Sitemaps {
    pub fn new(
        pool: DbPool,
        pages: Arc<Pages>,
        emails: Arc<Emails>,
        fonts: Arc<Fonts>,
        colors: Arc<Colors>,
        layouts: Arc<Layouts>,
    ) -> Self {
        Self {
            pool,
            pages,
            emails,
            fonts,
            colors,
            layouts,
        }
    }

    pub async fn create(&self, organization_id: &Id, branch: &str) -> Result<Id, sqlx::Error> {
        sqlx::query("insert into sitemaps (organization_id, branch) values ($1, $2)")
            .bind(organization_id)
            .bind(branch)
            .execute(&self.pool)
            .await
            .map(|r| r.last_insert_rowid())
    }

    pub async fn create_with_default_content(
        &self,
        organization_id: &Id,
        branch: &str,
    ) -> Result<Id, sqlx::Error> {
        let sitemap_id = self.create(&organization_id, branch).await?;

        // create default pages
        self.pages
            .create(&sitemap_id, "/", "inicio", "Inicio")
            .await?;

        // create default emails
        self.emails.create(&sitemap_id, "invitation").await?;

        // create default layouts
        self.layouts.create(&sitemap_id, "default").await?;

        Ok(sitemap_id)
    }

    pub async fn get_one_by_branch(
        &self,
        organization_id: &Id,
        branch: &str,
    ) -> Result<Sitemap, sqlx::Error> {
        sqlx::query_as::<_, Sitemap>(
            "select * from sitemaps where organization_id = $1 and branch = $2",
        )
        .bind(organization_id)
        .bind(branch)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_one_by_branch_optional(
        &self,
        organization_id: &Id,
        branch: &str,
    ) -> Result<Option<Sitemap>, sqlx::Error> {
        sqlx::query_as::<_, Sitemap>(
            "select * from sitemaps where organization_id = $1 and branch = $2",
        )
        .bind(organization_id)
        .bind(branch)
        .fetch_optional(&self.pool)
        .await
    }
}
