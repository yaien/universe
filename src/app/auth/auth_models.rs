use crate::infra::ID;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::prelude::FromRow;

pub struct OAuthAccountInfo {
    pub email: String,
    pub name: String,
    pub provider_user_id: String,
    pub provider_name: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(FromRow, Clone)]
pub struct User {
    pub id: ID,
    pub email: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
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
