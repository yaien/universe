use chrono::{DateTime, Utc};

use crate::infra::{DbPool, Id};

pub struct Email {
    pub id: i64,
    pub sitemap_id: i64,
    pub name: String,
    pub subject: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct Emails {
    pool: DbPool,
}
impl Emails {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, sitemap_id: &Id, name: &str) -> Result<Id, sqlx::Error> {
        sqlx::query("insert into emails (sitemap_id, name) values ($1, $2)")
            .bind(sitemap_id)
            .bind(name)
            .execute(&self.pool)
            .await
            .map(|r| r.last_insert_rowid())
    }
}
