use std::sync::Arc;

use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;

use crate::{
    app::{Branch, Emails, Layouts, Pages, Sitemaps},
    infra::{DbPool, Id},
};

#[derive(FromRow, Clone)]
pub struct Organization {
    pub id: i64,
    pub hostname: String,
    pub url: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct Organizations {
    pool: DbPool,
    sitemaps: Arc<Sitemaps>,
    pages: Arc<Pages>,
    emails: Arc<Emails>,
    layouts: Arc<Layouts>,
}

impl Organizations {
    pub fn new(
        pool: DbPool,
        sitemaps: Arc<Sitemaps>,
        pages: Arc<Pages>,
        emails: Arc<Emails>,
        layouts: Arc<Layouts>,
    ) -> Self {
        Self {
            pool,
            sitemaps,
            pages,
            emails,
            layouts,
        }
    }

    pub async fn create(&self, url: &str, hostname: &str, title: &str) -> Result<Id, sqlx::Error> {
        let organization_id =
            sqlx::query("insert into organizations (url, hostname, title) values ($1, $2, $3)")
                .bind(url)
                .bind(hostname)
                .bind(title)
                .execute(&self.pool)
                .await
                .map(|r| r.last_insert_rowid())?;

        for branch in [Branch::MAIN, Branch::DRAFT] {
            let sitemap_id = self.sitemaps.create(&organization_id, branch).await?;
            self.pages.create(&sitemap_id, "/").await?;
            self.emails.create(&sitemap_id, "invitation").await?;
            self.layouts.create(&sitemap_id, "default").await?;
        }

        Ok(organization_id)
    }

    pub async fn get_one_by_host(&self, hostname: &str) -> Result<Organization, sqlx::Error> {
        sqlx::query_as::<_, Organization>("select * from organizations where hostname = $1")
            .bind(hostname)
            .fetch_one(&self.pool)
            .await
    }
}

#[cfg(test)]
mod tests {
    use sqlx::{Row, SqlitePool};

    use crate::app::Branch;

    use super::*;

    #[tokio::test]
    async fn test_create_organization() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        sqlx::migrate!().run(&pool).await.unwrap();

        let organizations = Organizations::new(
            pool.clone(),
            Arc::new(Sitemaps::new(pool.clone())),
            Arc::new(Pages::new(pool.clone())),
            Arc::new(Emails::new(pool.clone())),
            Arc::new(Layouts::new(pool.clone())),
        );

        let organization_id = organizations
            .create("http://localhost:3000", "localhost:3000", "Localhost")
            .await
            .unwrap();

        let organization_count: u64 =
            sqlx::query("select count(*) from organizations where hostname = $1")
                .bind("localhost:3000")
                .fetch_one(&pool)
                .await
                .unwrap()
                .get(0);

        assert_eq!(organization_count, 1);

        for branch in [Branch::MAIN, Branch::DRAFT] {
            let sitemap_id: Id =
                sqlx::query("select id from sitemaps where organization_id = $1 and branch = $2")
                    .bind(organization_id)
                    .bind(branch)
                    .fetch_one(&pool)
                    .await
                    .unwrap()
                    .get(0);

            sqlx::query("select id from pages where sitemap_id = $1 and path = '/'")
                .bind(sitemap_id)
                .fetch_one(&pool)
                .await
                .unwrap();

            sqlx::query("select id from layouts where sitemap_id = $1 and name = 'default'")
                .bind(sitemap_id)
                .fetch_one(&pool)
                .await
                .unwrap();

            sqlx::query("select id from emails where sitemap_id = $1 and name = 'invitation'")
                .bind(sitemap_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        }
    }
}
