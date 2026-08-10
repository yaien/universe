use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;

use crate::infra::{DbPool, Id};

#[derive(FromRow, Clone)]
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

    pub async fn get_by_id(&self, sitemap_id: &Id, id: &Id) -> Result<Email, sqlx::Error> {
        sqlx::query_as::<_, Email>("select * from emails where sitemap_id = $1 and id = $2")
            .bind(sitemap_id)
            .bind(id)
            .fetch_one(&self.pool)
            .await
    }

    pub async fn get_by_sitemap_id(&self, sitemap_id: &Id) -> Result<Vec<Email>, sqlx::Error> {
        sqlx::query_as::<_, Email>("select * from emails where sitemap_id = $1")
            .bind(sitemap_id)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn update_body(
        &self,
        sitemap_id: &Id,
        email_id: &Id,
        body: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("update emails set body = $1 where sitemap_id = $2 and id = $3")
            .bind(body)
            .bind(sitemap_id)
            .bind(email_id)
            .execute(&self.pool)
            .await
            .map(|_| ())
    }
}
