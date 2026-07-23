use crate::infra::ID;
use anyhow::Result;
use sqlx::SqlitePool;

pub async fn create_organization<'b>(
    pool: &SqlitePool,
    url: String,
    hostname: String,
    title: String,
) -> Result<ID> {
    let organization_id =
        sqlx::query("insert into organizations (url, hostname, title) values ($1, $2, $3)")
            .bind(&url)
            .bind(&hostname)
            .bind(&title)
            .execute(pool)
            .await?
            .last_insert_rowid();

    create_sitemap_with_defaults(pool, organization_id, "active".to_string()).await?;
    create_sitemap_with_defaults(pool, organization_id, "draft".to_string()).await?;

    Ok(organization_id)
}

pub async fn create_sitemap_with_defaults<'b>(
    pool: &SqlitePool,
    organization_id: ID,
    name: String,
) -> Result<ID> {
    let sitemap_id = sqlx::query!(
        "insert into sitemaps (organization_id, name) values ($1, $2)",
        organization_id,
        &name,
    )
    .execute(pool)
    .await?
    .last_insert_rowid();

    create_page(pool, sitemap_id, "/".to_string()).await?;
    create_layout(pool, sitemap_id, "default".to_string()).await?;
    create_email(pool, sitemap_id, "invitation".to_string()).await?;

    Ok(sitemap_id)
}

pub async fn create_page<'a>(pool: &SqlitePool, sitemap_id: ID, path: String) -> Result<ID> {
    let id = sqlx::query("insert into pages (sitemap_id, path) values ($1, $2)")
        .bind(sitemap_id)
        .bind(path)
        .execute(pool)
        .await?
        .last_insert_rowid();

    Ok(id)
}

pub async fn create_layout<'a>(pool: &SqlitePool, sitemap_id: ID, name: String) -> Result<ID> {
    let id = sqlx::query("insert into layouts (sitemap_id, name) values ($1, $2)")
        .bind(sitemap_id)
        .bind(name)
        .execute(pool)
        .await?
        .last_insert_rowid();

    Ok(id)
}

pub async fn create_email<'a>(pool: &SqlitePool, sitemap_id: ID, name: String) -> Result<ID> {
    let id = sqlx::query("insert into emails (sitemap_id, name) values ($1, $2)")
        .bind(sitemap_id)
        .bind(name)
        .execute(pool)
        .await?
        .last_insert_rowid();

    Ok(id)
}
