use crate::infra::config::Config;
use anyhow::{Context, Result};
use oauth2::{
    EndpointNotSet, EndpointSet, RedirectUrl,
    basic::BasicClient,
    reqwest::{Client as OAuth2Client, ClientBuilder, redirect::Policy},
};
use reqwest::Client as ReqwestClient;
use sqlx::{
    ConnectOptions, SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};

pub type ID = i64;

pub type DBConnection = sqlx::SqliteConnection;

pub type DbPool = SqlitePool;

pub type GoogleClient =
    BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>;

pub struct Monolith {
    pub config: Config,
    pub pool: SqlitePool,
    pub oauth2_google_client: GoogleClient,
    pub oauth2_client: OAuth2Client,
    pub http_client: ReqwestClient,
}

impl Monolith {
    pub async fn build(config: Config) -> Result<Self> {
        // Sqlite Connection
        let options = SqliteConnectOptions::from_url(&config.database_url)?
            .journal_mode(SqliteJournalMode::Wal)
            .foreign_keys(true);

        let pool = SqlitePoolOptions::new()
            .connect_with(options)
            .await
            .context("failed initializing database")?;

        // Google OAuth2
        let mut redirect_uri = config.server_url.clone();
        redirect_uri.set_path("/auth/google/callback");
        let redirect_uri = RedirectUrl::new(redirect_uri.to_string())?;

        println!("Redirect uri {}", redirect_uri);

        let oauth2_google_client = BasicClient::new(config.google_client_id.clone())
            .set_client_secret(config.google_client_secret.clone())
            .set_auth_uri(config.google_auth_url.clone())
            .set_token_uri(config.google_token_url.clone())
            .set_redirect_uri(redirect_uri);

        let oauth2_client = ClientBuilder::new().redirect(Policy::none()).build()?;

        let http_client = ReqwestClient::new();

        Ok(Monolith {
            config,
            pool,
            oauth2_google_client,
            oauth2_client,
            http_client,
        })
    }

    pub async fn build_from_env() -> Result<Self> {
        let config = Config::new_from_env();
        Self::build(config).await
    }
}
