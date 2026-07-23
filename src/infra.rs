use anyhow::{Context, Result};
use axum::Router;
use dotenv::dotenv;
use sqlx::{
    Sqlite, SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};
use std::{env, str::FromStr};

#[derive(Debug)]
pub struct Config {
    pub database_url: String,
    pub server_addr: String,
    pub server_url: String,
}

impl Config {
    pub fn new_from_env() -> Self {
        dotenv().ok();
        Config {
            database_url: env::var("DATABASE_URL").expect("missing database url"),
            server_addr: env::var("SERVER_ADDR").unwrap_or(String::from("0.0.0.0:3300")),
            server_url: env::var("SERVER_URL").unwrap_or(String::from("http://localhost:3300")),
        }
    }
}

#[derive(Debug)]
pub struct Monolith {
    pub config: Config,
    pub pool: SqlitePool,
    pub router: Router,
}

impl Monolith {
    pub async fn build(config: Config) -> Result<Self> {
        let options = SqliteConnectOptions::from_str(&config.database_url)?
            .journal_mode(SqliteJournalMode::Wal)
            .foreign_keys(true);

        let pool = SqlitePoolOptions::new()
            .connect_with(options)
            .await
            .context("failed initializing database")?;

        let router = Router::new();

        Ok(Monolith {
            config,
            pool,
            router,
        })
    }

    pub async fn build_from_env() -> Result<Self> {
        let config = Config::new_from_env();
        Self::build(config).await
    }
}

pub type ID = i64;
pub type DB = Sqlite;
