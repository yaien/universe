use std::collections::HashMap;

use sqlx::QueryBuilder;
use sqlx::prelude::FromRow;

use crate::infra::{DbPool, Id};

#[derive(FromRow)]
pub struct Font {
    pub id: Id,

    pub family: String,

    #[sqlx(json)]
    pub subsets: Vec<String>,

    #[sqlx(json)]
    pub variants: Vec<String>,

    #[sqlx(json)]
    pub files: HashMap<String, String>,
}

#[derive(FromRow)]
pub struct SitemapFont {
    pub id: Id,
    pub tag: String,
    pub font_id: Id,
    pub font_family: String,

    #[sqlx(json)]
    pub font_variants: Vec<String>,

    #[sqlx(json)]
    pub font_files: HashMap<String, String>,
}

pub struct Fonts {
    pool: DbPool,
}

impl Fonts {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn find(
        &self,
        query: Option<String>,
        limit: Option<u16>,
        offset: Option<u16>,
    ) -> Result<Vec<Font>, sqlx::Error> {
        let limit = limit.filter(|l| *l <= 30).unwrap_or(10);
        let offset = offset.unwrap_or(0);

        let mut st = QueryBuilder::new("select * from fonts");

        if let Some(query) = query {
            st.push(" where family like ")
                .push_bind(format!("%{query}%"));
        };

        st.push(" limit ").push_bind(limit);
        st.push(" offset ").push_bind(offset);

        st.build_query_as::<Font>().fetch_all(&self.pool).await
    }

    pub async fn get_one(&self, id: &Id) -> Result<Font, sqlx::Error> {
        sqlx::query_as::<_, Font>("select * from fonts where id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
    }

    pub async fn create_sitemap_font(
        &self,
        sitemap_id: &Id,
        font_id: &Id,
        tag: &str,
    ) -> Result<Id, sqlx::Error> {
        sqlx::query("insert into sitemaps_fonts(sitemap_id, font_id, tag) values ($1, $2, $3)")
            .bind(sitemap_id)
            .bind(font_id)
            .bind(tag)
            .execute(&self.pool)
            .await
            .map(|r| r.last_insert_rowid())
    }

    pub async fn get_by_sitemap_id(
        &self,
        sitemap_id: &Id,
    ) -> Result<Vec<SitemapFont>, sqlx::Error> {
        sqlx::query_as::<_, SitemapFont>(
            r#"
            select sf.id, sf.tag, f.id as font_id, f.family as font_family, f.variants as font_variants, f.files as font_files from sitemaps_fonts sf
            join fonts f on f.id = sf.font_id
            where sitemap_id = $1
            "#,
        )
        .bind(sitemap_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_one_sitemap_font(
        &self,
        sitemap_id: &Id,
        id: &Id,
    ) -> Result<SitemapFont, sqlx::Error> {
        sqlx::query_as::<_, SitemapFont>(
                    r#"
                    select sf.id, sf.tag, f.id as font_id, f.family as font_family, f.variants as font_variants, f.files as font_files from sitemaps_fonts sf
                    join fonts f on f.id = sf.font_id
                    where sf.sitemap_id = $1 and sf.id = $2
                    "#,
                )
                .bind(sitemap_id)
                .bind(id)
                .fetch_one(&self.pool)
                .await
    }

    pub async fn update_sitemap_font(
        &self,
        id: &Id,
        sitemap_id: &Id,
        font_id: &Id,
        tag: &str,
    ) -> Result<(), sqlx::Error> {
        let result = sqlx::query(
            "update sitemaps_fonts set font_id = $1, tag = $2 where id = $3 and sitemap_id = $4",
        )
        .bind(font_id)
        .bind(tag)
        .bind(id)
        .bind(sitemap_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        Ok(())
    }

    pub async fn delete_by_sitemap_id(&self, sitemap_id: &Id) -> Result<(), sqlx::Error> {
        sqlx::query("delete from sitemaps_fonts where sitemap_id = $1")
            .bind(sitemap_id)
            .execute(&self.pool)
            .await
            .map(|_| ())
    }
}
