use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;

use crate::infra::{DbPool, ID};

#[derive(FromRow, Clone)]
pub struct User {
    pub id: ID,
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

    pub async fn get_one_by_id(&self, id: &ID) -> Result<User, sqlx::Error> {
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
}
