use sqlx::prelude::FromRow;

use crate::infra::{DbPool, Id};

#[derive(FromRow, Clone)]
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

    pub async fn create_from(&self, layout: &Layout) -> Result<Id, sqlx::Error> {
        sqlx::query(
            "insert into layouts (sitemap_id, name, html, css, js) values ($1, $2, $3, $4, $5)",
        )
        .bind(&layout.sitemap_id)
        .bind(&layout.name)
        .bind(&layout.html)
        .bind(&layout.css)
        .bind(&layout.js)
        .execute(&self.pool)
        .await
        .map(|r| r.last_insert_rowid())
    }

    pub async fn get_by_id(&self, sitemap_id: &Id, id: &Id) -> Result<Layout, sqlx::Error> {
        sqlx::query_as::<_, Layout>("select * from layouts where sitemap_id = $1 and id = $2")
            .bind(sitemap_id)
            .bind(id)
            .fetch_one(&self.pool)
            .await
    }

    pub async fn get_by_sitemap_id(&self, sitemap_id: &Id) -> Result<Vec<Layout>, sqlx::Error> {
        sqlx::query_as::<_, Layout>("select * from layouts where sitemap_id = $1")
            .bind(sitemap_id)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn update_html(
        &self,
        sitemap_id: &Id,
        layout_id: &Id,
        html: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("update layouts set html = $1 where sitemap_id = $2 and id = $3")
            .bind(html)
            .bind(sitemap_id)
            .bind(layout_id)
            .execute(&self.pool)
            .await
            .map(|_| ())
    }

    pub async fn update_css(
        &self,
        sitemap_id: &Id,
        layout_id: &Id,
        css: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("update layouts set css = $1 where sitemap_id = $2 and id = $3")
            .bind(css)
            .bind(sitemap_id)
            .bind(layout_id)
            .execute(&self.pool)
            .await
            .map(|_| ())
    }

    pub async fn update_js(
        &self,
        sitemap_id: &Id,
        layout_id: &Id,
        js: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("update layouts set js = $1 where sitemap_id = $2 and id = $3")
            .bind(js)
            .bind(sitemap_id)
            .bind(layout_id)
            .execute(&self.pool)
            .await
            .map(|_| ())
    }

    pub async fn delete_by_sitemap_id(&self, sitemap_id: &Id) -> Result<(), sqlx::Error> {
        sqlx::query("delete from layouts where sitemap_id = $1")
            .bind(sitemap_id)
            .execute(&self.pool)
            .await
            .map(|_| ())
    }
}
