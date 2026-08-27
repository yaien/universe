use sqlx::prelude::FromRow;

use crate::app;
use crate::infra::{DbPool, Id};

#[derive(FromRow, Clone)]
pub struct Page {
    pub id: Id,
    pub sitemap_id: Id,
    pub layout_id: Option<Id>,
    pub path: String,
    pub name: String,
    pub title: String,
    pub og_image_file_id: Option<Id>,
    pub og_type: String,
    pub og_description: String,
    pub html: String,
    pub css: String,
    pub js: String,
}

pub struct PageInfo {
    pub sitemap_id: Id,
    pub page_id: Id,
    pub path: String,
    pub name: String,
    pub title: String,
    pub layout_id: Option<Id>,
    pub og_type: String,
    pub og_description: String,
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
    ) -> Result<Page, sqlx::Error> {
        sqlx::query_as::<_, Page>(
            "insert into pages (sitemap_id, path, name, title) values ($1, $2, $3, $4) returning *",
        )
        .bind(sitemap_id)
        .bind(path)
        .bind(name)
        .bind(title)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn create_from(&self, page: &Page) -> Result<Id, sqlx::Error> {
        sqlx::query(r#"
            insert into pages (sitemap_id, layout_id, path, name, title, og_image, og_type, og_description, html, css, js)
                values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#)
            .bind(&page.sitemap_id)
            .bind(&page.layout_id)
            .bind(&page.path)
            .bind(&page.name)
            .bind(&page.title)
            .bind(&page.og_image_file_id)
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

    pub async fn get_by_path(&self, sitemap_id: &Id, path: &str) -> Result<Page, sqlx::Error> {
        sqlx::query_as::<_, Page>("select * from pages where sitemap_id = $1 and $2 like path order by length(path) desc limit 1")
            .bind(sitemap_id)
            .bind(path)
            .fetch_one(&self.pool)
            .await
    }

    pub async fn get_oldest(&self, sitemap_id: &Id) -> Result<Page, sqlx::Error> {
        sqlx::query_as::<_, Page>(
            "select * from pages where sitemap_id = $1 order by id asc limit 1",
        )
        .bind(sitemap_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_by_sitemap_id(&self, sitemap_id: &Id) -> Result<Vec<Page>, sqlx::Error> {
        sqlx::query_as::<_, Page>("select * from pages where sitemap_id = $1")
            .bind(sitemap_id)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn update_info(&self, info: &PageInfo) -> Result<(), sqlx::Error> {
        sqlx::query(r#"
            update pages set path = $1, name = $2, title = $3, layout_id = $4, og_description = $5, og_image = $6, og_type = $7
                where sitemap_id = $8 and id = $9
        "#)
            .bind(&info.path)
            .bind(&info.name)
            .bind(&info.title)
            .bind(&info.layout_id)
            .bind(&info.og_description)
            .bind(&info.og_type)
            .bind(&info.sitemap_id)
            .bind(&info.page_id)
            .execute(&self.pool)
            .await
            .map(|_| ())
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

    pub async fn delete_one_by_sitemap_id(
        &self,
        sitemap_id: &Id,
        page_id: &Id,
    ) -> Result<(), app::AppError> {
        let count: i64 = sqlx::query_scalar("select count(*) from pages where sitemap_id = $1")
            .bind(sitemap_id)
            .fetch_one(&self.pool)
            .await?;

        if count == 1 {
            return Err(app::AppError::Message("cannot delete the only page".into()));
        }

        sqlx::query("delete from pages where sitemap_id = $1 and id = $2")
            .bind(sitemap_id)
            .bind(page_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
