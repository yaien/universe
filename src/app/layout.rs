use crate::infra::{DbPool, Id};

pub struct Layout {
    pub id: i64,
    pub sitemap_id: i64,
    pub name: String,
    pub html: String,
    pub css: String,
    pub js: String,
}

pub struct Layouts {
    pool: DbPool,
}

impl Layouts {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, sitemap_id: &Id, name: &str) -> Result<Id, sqlx::Error> {
        sqlx::query("insert into layouts (sitemap_id, name) values ($1, $2)")
            .bind(sitemap_id)
            .bind(name)
            .execute(&self.pool)
            .await
            .map(|r| r.last_insert_rowid())
    }
}
