use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;

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
    pub pool: DbPool,
}

impl Sitemaps {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, organization_id: &Id, branch: &str) -> Result<Id, sqlx::Error> {
        sqlx::query("insert into sitemaps (organization_id, branch) values ($1, $2)")
            .bind(organization_id)
            .bind(branch)
            .execute(&self.pool)
            .await
            .map(|r| r.last_insert_rowid())
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
}
