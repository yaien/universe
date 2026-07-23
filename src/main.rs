mod app;
mod cmd;
mod infra;

use anyhow::{Context, Result};
use app::organization;
use cmd::{Args, Command, Parser};
use infra::Monolith;
use sqlx::migrate;
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    let cmd = Args::parse();
    match &cmd.command {
        Command::Serve => {
            let mono = Monolith::build_from_env()
                .await
                .context("failed initializing monolith")?;

            let listener = TcpListener::bind(mono.config.server_addr.clone())
                .await
                .context("failed tcp bind")?;

            tracing_subscriber::fmt::init();

            migrate!()
                .run(&mono.pool)
                .await
                .context("Migration run failed")?;

            info!("Migrations run successfully");

            info!("Server listening on {}", mono.config.server_url);

            axum::serve(listener, mono.router)
                .await
                .context("failed starting server")?;
        }
        Command::CreateOrganization {
            url,
            hostname,
            title,
        } => {
            let mono = Monolith::build_from_env()
                .await
                .context("failed initializing monolith")?;

            organization::create_organization(
                &mono.pool,
                url.to_string(),
                hostname.to_string(),
                title.to_string(),
            )
            .await
            .context("failed creating organization")?;

            info!("Organization created successfully");
        }
    }

    Ok(())
}
