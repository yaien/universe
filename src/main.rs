mod app;
mod cmd;
mod infra;
mod web;

use anyhow::{Context, Result};
use app::organization;
use cmd::{Args, Command, Parser};
use sqlx::migrate;
use tokio::net::TcpListener;
use tracing::info;

use infra::Monolith;

#[tokio::main]
async fn main() -> Result<()> {
    let cmd = Args::parse();
    match &cmd.command {
        Some(Command::CreateOrganization {
            url,
            hostname,
            title,
        }) => {
            let mono = Monolith::build_from_env()
                .await
                .context("failed initializing monolith")?;

            let mut tx = mono
                .pool
                .begin()
                .await
                .context("failed starting transaction")?;

            organization::create_organization(&mut tx, url, hostname, title)
                .await
                .context("failed creating organization")?;

            tx.commit().await.context("failed committing transaction")?;

            info!("Organization created successfully");

            return Ok(());
        }
        None => {
            let mono = Monolith::build_from_env()
                .await
                .context("failed initializing monolith")?;

            tracing_subscriber::fmt::init();

            migrate!()
                .run(&mono.pool)
                .await
                .context("Migration run failed")?;

            info!("Migrations run successfully");

            let listener = TcpListener::bind(&mono.config.server_addr)
                .await
                .context("failed tcp bind")?;

            let router = web::new_router(mono.clone());

            info!("Server listening on {}", &mono.config.server_url);

            axum::serve(listener, router)
                .await
                .context("failed starting server")?;

            Ok(())
        }
    }
}
