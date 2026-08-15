use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;

use crate::infra::{DbPool, Id};

#[derive(FromRow, Clone)]
pub struct Page {
    pub id: Id,
    pub sitemap_id: Id,
    pub layout_id: Option<Id>,
    pub path: String,
    pub name: String,
    pub title: String,
    pub og_image: String,
    pub og_type: String,
    pub og_description: String,
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

    pub async fn create(
        &self,
        sitemap_id: &Id,
        path: &str,
        name: &str,
        title: &str,
    ) -> Result<Id, sqlx::Error> {
        sqlx::query("insert into pages (sitemap_id, path, name, title) values ($1, $2, $3, $4)")
            .bind(sitemap_id)
            .bind(path)
            .bind(name)
            .bind(title)
            .execute(&self.pool)
            .await
            .map(|r| r.last_insert_rowid())
    }

    pub async fn create_from(&self, page: &Page) -> Result<Id, sqlx::Error> {
        sqlx::query("insert into pages (sitemap_id, layout_id, path, name, title, og_image, og_type, og_description, html, css, js) values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)")
            .bind(&page.sitemap_id)
            .bind(&page.layout_id)
            .bind(&page.path)
            .bind(&page.name)
            .bind(&page.title)
            .bind(&page.og_image)
            .bind(&page.og_type)
            .bind(&page.og_description)
            .bind(&page.html)
            .bind(&page.css)
            .bind(&page.js)
            .execute(&self.pool)
            .await
            .map(|r| r.last_insert_rowid())
    }

    pub async fn get_by_id(&self, sitemap_id: &Id, page_id: &Id) -> Result<Page, sqlx::Error> {
        sqlx::query_as::<_, Page>("select * from pages where sitemap_id = $1 and id = $2")
            .bind(sitemap_id)
            .bind(page_id)
            .fetch_one(&self.pool)
            .await
    }

    pub async fn get_by_sitemap_id(&self, sitemap_id: &Id) -> Result<Vec<Page>, sqlx::Error> {
        sqlx::query_as::<_, Page>("select * from pages where sitemap_id = $1")
            .bind(sitemap_id)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn update_html(
        &self,
        sitemap_id: &Id,
        page_id: &Id,
        html: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("update pages set html = $1 where sitemap_id = $2 and id = $3")
            .bind(html)
            .bind(sitemap_id)
            .bind(page_id)
            .execute(&self.pool)
            .await
            .map(|_| ())
    }

    pub async fn update_css(
        &self,
        sitemap_id: &Id,
        page_id: &Id,
        css: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("update pages set css = $1 where sitemap_id = $2 and id = $3")
            .bind(css)
            .bind(sitemap_id)
            .bind(page_id)
            .execute(&self.pool)
            .await
            .map(|_| ())
    }

    pub async fn update_js(
        &self,
        sitemap_id: &Id,
        page_id: &Id,
        js: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("update pages set js = $1 where sitemap_id = $2 and id = $3")
            .bind(js)
            .bind(sitemap_id)
            .bind(page_id)
            .execute(&self.pool)
            .await
            .map(|_| ())
    }

    pub async fn delete_by_sitemap_id(&self, sitemap_id: &Id) -> Result<(), sqlx::Error> {
        sqlx::query("delete from pages where sitemap_id = $1")
            .bind(sitemap_id)
            .execute(&self.pool)
            .await
            .map(|_| ())
    }
}
