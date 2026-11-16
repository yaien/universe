use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::prelude::FromRow;

use crate::infra::{DbPool, Id};

#[derive(FromRow, Clone, Serialize)]
pub struct User {
    pub id: Id,
    pub email: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

pub struct Users {
    pool: DbPool,
}

impl Users {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, name: &str, email: &str) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>("INSERT INTO users (name, email) VALUES ($1, $2) RETURNING *")
            .bind(name)
            .bind(email)
            .fetch_one(&self.pool)
            .await
    }

    pub async fn get_one_by_id(&self, id: &Id) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
    }

    pub async fn get_one_by_email(&self, email: &str) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_one(&self.pool)
            .await
    }

    pub async fn get_by_organization_id(&self, org_id: &Id) -> Result<Vec<User>, sqlx::Error> {
        sqlx::query_as("SELECT u.* FROM users u JOIN organization_users org on org.user_id = u.id where org.organization_id = $1")
            .bind(org_id)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn associate_user_with_organization(
        &self,
        user_id: &Id,
        org_id: &Id,
    ) -> Result<(), sqlx::Error> {
        let association_exist: bool = sqlx::query_scalar(
            "select exists(select 1 from organization_users where user_id = $1 and organization_id = $2)",
        )
        .bind(user_id)
        .bind(org_id)
        .fetch_one(&self.pool)
        .await?;

        if !association_exist {
            sqlx::query(
                "insert into organization_users (user_id, organization_id) values ($1, $2)",
            )
            .bind(user_id)
            .bind(org_id)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }
}
