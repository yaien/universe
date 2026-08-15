use sqlx::prelude::FromRow;

use crate::infra::{DbPool, Id};

#[derive(FromRow)]
pub struct Color {
    pub id: Id,
    pub sitemap_id: Id,
    pub tag: String,
    pub value: String,
}

pub struct Colors {
    pool: DbPool,
}

impl Colors {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_sitemap_id(&self, sitemap_id: &Id) -> Result<Vec<Color>, sqlx::Error> {
        sqlx::query_as::<_, Color>("select * from colors where sitemap_id = $1")
            .bind(sitemap_id)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn update(
        &self,
        sitemap_id: &Id,
        id: &Id,
        tag: &str,
        value: &str,
    ) -> Result<(), sqlx::Error> {
        let result =
            sqlx::query("update colors set tag = $1, value = $2 where sitemap_id = $3 and id = $4")
                .bind(tag)
                .bind(value)
                .bind(sitemap_id)
                .bind(id)
                .execute(&self.pool)
                .await?;

        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        Ok(())
    }

    pub async fn create(&self, sitemap_id: &Id) -> Result<Color, sqlx::Error> {
        let colors = self.get_by_sitemap_id(sitemap_id).await?;
        let mut new_tag_index = colors.len();
        let mut new_tag_name = format!("black-{new_tag_index}").to_string();
        while colors.iter().any(|c| c.tag == new_tag_name) {
            new_tag_index += 1;
            new_tag_name = format!("black-{}", new_tag_index);
        }

        sqlx::query_as::<_, Color>(
            "insert into colors(sitemap_id, tag, value) values ($1, $2, $3) returning *",
        )
        .bind(sitemap_id)
        .bind(new_tag_name)
        .bind("#000")
        .fetch_one(&self.pool)
        .await
    }

    pub async fn create_from(&self, color: &Color) -> Result<Color, sqlx::Error> {
        sqlx::query_as::<_, Color>(
            "insert into colors(sitemap_id, tag, value) values ($1, $2, $3) returning *",
        )
        .bind(&color.sitemap_id)
        .bind(&color.tag)
        .bind(&color.value)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn delete(&self, sitemap_id: &Id, id: &Id) -> Result<(), sqlx::Error> {
        sqlx::query("delete from colors where sitemap_id = $1 and id = $2")
            .bind(sitemap_id)
            .bind(id)
            .execute(&self.pool)
            .await
            .map(|_| ())
    }

    pub async fn delete_by_sitemap_id(&self, sitemap_id: &Id) -> Result<(), sqlx::Error> {
        sqlx::query("delete from colors where sitemap_id = $1")
            .bind(sitemap_id)
            .execute(&self.pool)
            .await
            .map(|_| ())
    }
}
