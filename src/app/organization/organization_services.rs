use crate::app::organization::{Branch, Organization};
use crate::infra::{DBConnection, ID};
use anyhow::Result;

pub async fn create_organization(
    conn: &mut DBConnection,
    url: &str,
    hostname: &str,
    title: &str,
) -> Result<ID> {
    let organization_id =
        sqlx::query("insert into organizations (url, hostname, title) values ($1, $2, $3)")
            .bind(url)
            .bind(hostname)
            .bind(title)
            .execute(&mut *conn)
            .await?
            .last_insert_rowid();

    create_sitemap_with_defaults(conn, organization_id, Branch::MAIN).await?;
    create_sitemap_with_defaults(conn, organization_id, Branch::DRAFT).await?;

    Ok(organization_id)
}

pub async fn create_sitemap_with_defaults(
    conn: &mut DBConnection,
    organization_id: ID,
    branch: &str,
) -> Result<ID> {
    let sitemap_id = sqlx::query("insert into sitemaps (organization_id, branch) values ($1, $2)")
        .bind(organization_id)
        .bind(branch)
        .execute(&mut *conn)
        .await?
        .last_insert_rowid();

    create_page(&mut *conn, sitemap_id, "/".to_string()).await?;
    create_layout(&mut *conn, sitemap_id, "default".to_string()).await?;
    create_email(&mut *conn, sitemap_id, "invitation".to_string()).await?;

    Ok(sitemap_id)
}

pub async fn create_page(conn: &mut DBConnection, sitemap_id: ID, path: String) -> Result<ID> {
    let id = sqlx::query("insert into pages (sitemap_id, path) values ($1, $2)")
        .bind(sitemap_id)
        .bind(path)
        .execute(&mut *conn)
        .await?
        .last_insert_rowid();

    Ok(id)
}

pub async fn create_layout(conn: &mut DBConnection, sitemap_id: ID, name: String) -> Result<ID> {
    let id = sqlx::query("insert into layouts (sitemap_id, name) values ($1, $2)")
        .bind(sitemap_id)
        .bind(name)
        .execute(conn)
        .await?
        .last_insert_rowid();

    Ok(id)
}

pub async fn create_email(conn: &mut DBConnection, sitemap_id: ID, name: String) -> Result<ID> {
    let id = sqlx::query("insert into emails (sitemap_id, name) values ($1, $2)")
        .bind(sitemap_id)
        .bind(name)
        .execute(conn)
        .await?
        .last_insert_rowid();

    Ok(id)
}

pub async fn get_organization_by_host(conn: &mut DBConnection, host: &str) -> Result<Organization> {
    sqlx::query_as::<_, Organization>("select * from organizations where hostname = $1")
        .bind(host)
        .fetch_one(conn)
        .await
        .map_err(|e| anyhow::Error::from(e))
}

#[cfg(test)]
mod tests {
    use sqlx::{Connection, Row, SqliteConnection};

    use super::*;

    #[tokio::test]
    async fn test_create_organization() {
        let mut conn = SqliteConnection::connect(":memory:").await.unwrap();

        sqlx::migrate!().run(&mut conn).await.unwrap();

        let organization_id = create_organization(
            &mut conn,
            "http://localhost:3000",
            "localhost:3000",
            "Localhost",
        )
        .await
        .unwrap();

        let organization_count: u64 =
            sqlx::query("select count(*) from organizations where hostname = $1")
                .bind("localhost:3000")
                .fetch_one(&mut conn)
                .await
                .unwrap()
                .get(0);

        assert_eq!(organization_count, 1);

        for branch in [Branch::MAIN, Branch::DRAFT] {
            let sitemap_id: ID =
                sqlx::query("select id from sitemaps where organization_id = $1 and branch = $2")
                    .bind(organization_id)
                    .bind(branch)
                    .fetch_one(&mut conn)
                    .await
                    .unwrap()
                    .get(0);

            sqlx::query("select id from pages where sitemap_id = $1 and path = '/'")
                .bind(sitemap_id)
                .fetch_one(&mut conn)
                .await
                .unwrap();

            sqlx::query("select id from layouts where sitemap_id = $1 and name = 'default'")
                .bind(sitemap_id)
                .fetch_one(&mut conn)
                .await
                .unwrap();

            sqlx::query("select id from emails where sitemap_id = $1 and name = 'invitation'")
                .bind(sitemap_id)
                .fetch_one(&mut conn)
                .await
                .unwrap();
        }

        conn.close().await.unwrap();
    }
}
