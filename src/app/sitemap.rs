use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use sqlx::prelude::FromRow;

use crate::app::{self, Colors, Emails, Fonts, Layouts, Pages, bundle_css, bundle_js};
use crate::infra::{DbPool, Id};

pub struct Branch;

impl Branch {
    pub const MAIN: &'static str = "main";
    pub const DRAFT: &'static str = "draft";
}

#[derive(FromRow)]
pub struct Sitemap {
    pub id: Id,
    pub branch: String,
    pub favicon_file_id: Option<Id>,
}

pub struct Sitemaps {
    pool: DbPool,
    pages: Arc<Pages>,
    emails: Arc<Emails>,
    fonts: Arc<Fonts>,
    colors: Arc<Colors>,
    layouts: Arc<Layouts>,
}

impl Sitemaps {
    pub fn new(
        pool: DbPool,
        pages: Arc<Pages>,
        emails: Arc<Emails>,
        fonts: Arc<Fonts>,
        colors: Arc<Colors>,
        layouts: Arc<Layouts>,
    ) -> Self {
        Self {
            pool,
            pages,
            emails,
            fonts,
            colors,
            layouts,
        }
    }

    pub async fn create(&self, organization_id: &Id, branch: &str) -> Result<Id, sqlx::Error> {
        sqlx::query("insert into sitemaps (organization_id, branch) values ($1, $2)")
            .bind(organization_id)
            .bind(branch)
            .execute(&self.pool)
            .await
            .map(|r| r.last_insert_rowid())
    }

    pub async fn create_with_default_content(
        &self,
        organization_id: &Id,
        branch: &str,
    ) -> Result<Id, sqlx::Error> {
        let sitemap_id = self.create(&organization_id, branch).await?;

        // create default pages
        self.pages
            .create(&sitemap_id, "/", "inicio", "Inicio")
            .await?;

        // create default emails
        self.emails.create(&sitemap_id, "invitation").await?;

        // create default layouts
        self.layouts.create(&sitemap_id, "default").await?;

        Ok(sitemap_id)
    }

    pub async fn sync_branch(
        &self,
        org_id: &Id,
        from_sitemap: &Sitemap,
        to_branch: &str,
    ) -> Result<Id, sqlx::Error> {
        let to_sitemap_id = match self.get_one_by_branch_optional(org_id, to_branch).await? {
            Some(sitemap) => sitemap.id,
            None => self.create(org_id, to_branch).await?,
        };

        let layouts = self.layouts.get_by_sitemap_id(&from_sitemap.id).await?;
        let pages = self.pages.get_by_sitemap_id(&from_sitemap.id).await?;
        let fonts = self.fonts.get_by_sitemap_id(&from_sitemap.id).await?;
        let colors = self.colors.get_by_sitemap_id(&from_sitemap.id).await?;

        if to_branch == Branch::MAIN {
            let bundled_js = bundle_js(&layouts, &pages);
            let bundled_css = bundle_css(&layouts, &pages, &fonts, &colors);
            self.update(
                &to_sitemap_id,
                &bundled_js,
                &bundled_css,
                &from_sitemap.favicon_file_id,
            )
            .await?;
        }

        self.layouts.delete_by_sitemap_id(&to_sitemap_id).await?;
        self.pages.delete_by_sitemap_id(&to_sitemap_id).await?;
        self.fonts.delete_by_sitemap_id(&to_sitemap_id).await?;
        self.colors.delete_by_sitemap_id(&to_sitemap_id).await?;

        let mut layout_matches = HashMap::new();
        for mut layout in layouts {
            layout.sitemap_id = to_sitemap_id.clone();
            let created_layout_id = self.layouts.create_from(&layout).await?;
            layout_matches.insert(layout.id.clone(), created_layout_id);
        }

        for mut page in pages {
            page.sitemap_id = to_sitemap_id.clone();

            page.layout_id = page
                .layout_id
                .map(|id| layout_matches.get(&id).copied())
                .flatten();

            self.pages.create_from(&page).await?;
        }

        for font in fonts {
            self.fonts
                .create_sitemap_font(&to_sitemap_id, &font.font_id, &font.tag)
                .await?;
        }

        for mut color in colors {
            color.sitemap_id = to_sitemap_id.clone();
            self.colors.create_from(&color).await?;
        }

        Ok(to_sitemap_id)
    }

    pub async fn get_one_by_branch(
        &self,
        organization_id: &Id,
        branch: &str,
    ) -> Result<Sitemap, sqlx::Error> {
        sqlx::query_as::<_, Sitemap>(
            "select * from sitemaps where organization_id = $1 and branch = $2",
        )
        .bind(organization_id)
        .bind(branch)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_one_by_branch_optional(
        &self,
        organization_id: &Id,
        branch: &str,
    ) -> Result<Option<Sitemap>, sqlx::Error> {
        sqlx::query_as::<_, Sitemap>(
            "select * from sitemaps where organization_id = $1 and branch = $2",
        )
        .bind(organization_id)
        .bind(branch)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn get_drafts_by_organization_id(
        &self,
        org_id: &Id,
    ) -> Result<Vec<Sitemap>, sqlx::Error> {
        sqlx::query_as::<_, Sitemap>(
            "select * from sitemaps where organization_id = $1 and branch != $2",
        )
        .bind(org_id)
        .bind(Branch::MAIN)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn delete_one_by_organization_id(
        &self,
        branch: &str,
        org_id: &Id,
    ) -> app::Result<()> {
        if branch == Branch::MAIN || branch == Branch::DRAFT {
            return Err(app::AppError::Message(
                "cant delete main or draft branch".to_string(),
            ));
        }

        sqlx::query("delete from sitemaps where organization_id = $1 and branch = $2")
            .bind(org_id)
            .bind(branch)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn update(
        &self,
        sitemap_id: &Id,
        bundled_js: &str,
        bundled_css: &str,
        favicon_file_id: &Option<Id>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("update sitemaps set bundled_js = $1, bundled_css = $2, favicon_file_id = $3 where id = $4")
            .bind(bundled_js)
            .bind(bundled_css)
            .bind(favicon_file_id)
            .bind(sitemap_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update_favicon_file_id(
        &self,
        sitemap_id: &Id,
        file_id: &Id,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("update sitemaps set favicon_file_id = $1 where id = $2")
            .bind(file_id)
            .bind(sitemap_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_bundled_css(&self, org_id: &Id) -> Result<String, sqlx::Error> {
        sqlx::query_scalar(
            "select bundled_css from sitemaps where organization_id = $1 and branch = $2",
        )
        .bind(org_id)
        .bind(Branch::MAIN)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_bundled_js(&self, org_id: &Id) -> Result<String, sqlx::Error> {
        sqlx::query_scalar(
            "select bundled_js from sitemaps where organization_id = $1 and branch = $2",
        )
        .bind(org_id)
        .bind(Branch::MAIN)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_favicon_file_id(&self, org_id: &Id) -> Result<Option<Id>, sqlx::Error> {
        sqlx::query_scalar(
            "select favicon_file_id from sitemaps where organization_id = $1 and branch = $2",
        )
        .bind(org_id)
        .bind(Branch::MAIN)
        .fetch_one(&self.pool)
        .await
    }
}
