use chrono::{DateTime, Utc};

use crate::infra::{DbPool, Id};

pub struct Page {
    pub id: i64,
    pub sitemap_id: i64,
    pub layout_id: i64,
    pub path: String,
    pub html: String,
    pub css: String,
    pub js: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct Pages {
    pool: DbPool,
}

impl Pages {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, sitemap_id: &Id, path: &str) -> Result<Id, sqlx::Error> {
        sqlx::query("insert into pages (sitemap_id, path) values ($1, $2)")
            .bind(sitemap_id)
            .bind(path)
            .execute(&self.pool)
            .await
            .map(|r| r.last_insert_rowid())
    }
}
