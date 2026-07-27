use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::prelude::FromRow;

use crate::{
    app::{User, Users},
    infra::{DbPool, ID},
};

pub struct OAuthAccountInfo {
    pub email: String,
    pub name: String,
    pub provider_user_id: String,
    pub provider_name: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(FromRow)]
pub struct OAuthAccount {
    pub id: ID,
    pub user_id: ID,
    pub provider_name: String,
    pub provider_user_id: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(FromRow)]
pub struct OAuthState {
    pub id: ID,
    pub csrf_token: String,
    pub pkce_verifier: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub hostname: String,
    pub provider: String,
}

#[derive(Deserialize)]
pub struct GoogleUserInfo {
    pub sub: String,
    pub email: String,
    pub name: String,
}

pub struct Auth {
    pool: DbPool,
    users: Arc<Users>,
}

impl Auth {
    pub fn new(pool: DbPool, users: Arc<Users>) -> Self {
        Self { pool, users }
    }

    pub async fn create_oauth_state(&self, state: OAuthState) -> Result<ID, sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO oauth_states
               (csrf_token, pkce_verifier, created_at, expires_at, hostname, provider)
               VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(&state.csrf_token)
        .bind(&state.pkce_verifier)
        .bind(&state.created_at)
        .bind(&state.expires_at)
        .bind(&state.hostname)
        .bind(&state.provider)
        .execute(&self.pool)
        .await
        .map(|r| r.last_insert_rowid())
    }

    pub async fn get_oauth_state_by_csrf_token(
        &self,
        csrf_token: &str,
    ) -> Result<OAuthState, sqlx::Error> {
        sqlx::query_as::<_, OAuthState>(r#"SELECT * FROM oauth_states WHERE csrf_token = $1"#)
            .bind(csrf_token)
            .fetch_one(&self.pool)
            .await
    }

    pub async fn delete_oauth_state_by_csrf_token(
        &self,
        csrf_token: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(r#"DELETE FROM oauth_states WHERE csrf_token = $1"#)
            .bind(csrf_token)
            .execute(&self.pool)
            .await
            .and_then(|_| Ok(()))
    }

    pub async fn sync_oauth_account(&self, info: OAuthAccountInfo) -> Result<User, sqlx::Error> {
        let account = sqlx::query_as::<_, OAuthAccount>(
            "select * from oauth_accounts where provider_user_id = $1 and provider_name = $2",
        )
        .bind(&info.provider_user_id)
        .bind(&info.provider_name)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(account) = account {
            sqlx::query("update oauth_accounts set access_token = $1, refresh_token = $2, expires_at = $3 where id = $4")
                    .bind(&info.access_token)
                    .bind(&info.refresh_token)
                    .bind(&info.expires_at)
                    .bind(&account.id)
                    .execute(&self.pool)
                    .await?;

            return self.users.get_one_by_id(&account.user_id).await;
        }

        let user = match self.users.get_one_by_email(&info.email).await.ok() {
            Some(user) => user,
            None => self.users.create(&info.name, &info.email).await?,
        };

        sqlx::query(
            r#"insert into oauth_accounts
                (provider_user_id, provider_name, access_token, refresh_token, expires_at, user_id)
                values ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(&info.provider_user_id)
        .bind(&info.provider_name)
        .bind(&info.access_token)
        .bind(&info.refresh_token)
        .bind(&info.expires_at)
        .bind(&user.id)
        .execute(&self.pool)
        .await?;

        return Ok(user);
    }
}
