use std::sync::Arc;

use anyhow::Context;
use chrono::{DateTime, Utc};

use crate::app::{Roles, User};
use crate::infra::{DbPool, Id};

#[derive(sqlx::FromRow)]
pub struct Invitation {
    pub id: Id,
    pub user_email: String,
    pub organization_id: Id,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

pub struct Invitations {
    pool: DbPool,
}

impl Invitations {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        organization_id: &Id,
        user_email: &str,
        expires_at: &DateTime<Utc>,
    ) -> Result<Id, sqlx::Error> {
        let query = r#"
            insert into invitations (organization_id, user_email, expires_at)
            values ($1, $2, $3)
        "#;

        sqlx::query(query)
            .bind(organization_id)
            .bind(user_email)
            .bind(expires_at)
            .execute(&self.pool)
            .await
            .map(|r| r.last_insert_rowid())
    }

    pub async fn accept(
        &self,
        org_id: &Id,
        user_email: &str,
        user_id: &Id,
    ) -> Result<Id, anyhow::Error> {
        let query = r#"
            select * from invitations
            where organization_id = $1 and user_email = $2 and expires_at > $3
        "#;

        let invitation = sqlx::query_as::<_, Invitation>(query)
            .bind(org_id)
            .bind(user_email)
            .bind(Utc::now())
            .fetch_one(&self.pool)
            .await
            .context("invitation not found")?;

        let role_id = sqlx::query("insert into roles (user_id, organization_id) values ($1, $2)")
            .bind(user_id)
            .bind(org_id)
            .execute(&self.pool)
            .await
            .map(|r| r.last_insert_rowid())
            .context("failed inserting role")?;

        sqlx::query("delete from invitations where id = $1")
            .bind(invitation.id)
            .execute(&self.pool)
            .await
            .context("failed deleting invitation")?;

        Ok(role_id)
    }
}

#[cfg(test)]
mod tests {

    use chrono::Duration;
    use sqlx::migrate;

    use super::*;

    #[tokio::test]
    async fn test_create_and_accept_invitation() -> Result<(), anyhow::Error> {
        let pool = sqlx::SqlitePool::connect(":memory:").await?;
        migrate!().run(&pool).await?;
        let invitations = Invitations::new(pool.clone());
        let roles = Roles::new(pool.clone());

        let org_id =
            sqlx::query("insert into organizations (hostname, url, title) values ($1, $2, $3)")
                .bind("localhost")
                .bind("https://localhost")
                .bind("localhost")
                .execute(&pool)
                .await?
                .last_insert_rowid();

        let email = "user_1@email.com";

        let user_id = sqlx::query("insert into users (name, email) values ($1, $2)")
            .bind("user_1")
            .bind(email)
            .execute(&pool)
            .await?
            .last_insert_rowid();

        let expires_at = Utc::now() + Duration::hours(6);

        invitations.create(&org_id, email, &expires_at).await?;

        invitations.accept(&org_id, email, &user_id).await?;

        roles
            .get_one_by_org_id_and_user_id(&org_id, &user_id)
            .await?;

        Ok(())
    }
}
