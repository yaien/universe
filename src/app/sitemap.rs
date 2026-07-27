use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::infra::{DbPool, ID};

pub struct Branch {}

impl Branch {
    pub const MAIN: &'static str = "main";
    pub const DRAFT: &'static str = "draft";
}

pub struct Sitemap {
    pub id: i64,
    pub organization_id: i64,
    pub branch: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct Layout {
    pub id: i64,
    pub sitemap_id: i64,
    pub name: String,
    pub html: String,
    pub css: String,
    pub js: String,
}

pub struct Email {
    pub id: i64,
    pub sitemap_id: i64,
    pub name: String,
    pub subject: String,
    pub body: String,
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

    pub async fn create(&self, organization_id: &ID, branch: &str) -> Result<ID, sqlx::Error> {
        sqlx::query("insert into sitemaps (organization_id, branch) values ($1, $2)")
            .bind(organization_id)
            .bind(branch)
            .execute(&self.pool)
            .await
            .map(|r| r.last_insert_rowid())
    }
}
