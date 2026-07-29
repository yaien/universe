mod app;
mod cmd;
mod infra;
mod web;

use std::{fs::File, io::BufReader};

use actix_web::{App, HttpServer, web::Data};
use anyhow::{Context, Result};
use app::App as Service;
use chrono::{Duration, Utc};
use cmd::{Args, Command, Parser};
use rustls::crypto;
use sqlx::migrate;

use infra::Monolith;

#[tokio::main]
async fn main() -> Result<()> {
    let cmd = Args::parse();
    match &cmd.command {
        Some(Command::Create {
            url,
            hostname,
            title,
        }) => {
            let mono = Monolith::build_from_env()
                .await
                .context("failed initializing monolith")?;

            let service = Service::new(&mono);

            service
                .organizations
                .create(url, hostname, title)
                .await
                .context("failed creating organization")?;

            return Ok(());
        }
        Some(Command::Invite { email, hostname }) => {
            let mono = Monolith::build_from_env()
                .await
                .context("failed initializing monolith")?;

            let service = Service::new(&mono);

            let org = service
                .organizations
                .get_one_by_host(hostname)
                .await
                .context("organzation not found")?;

            let exp = Utc::now() + Duration::hours(3);

            service
                .invitations
                .create(&org.id, email, &exp)
                .await
                .context("failed inviting user")?;

            return Ok(());
        }
        None => {
            let mono = Monolith::build_from_env()
                .await
                .context("failed initializing monolith")?;

            migrate!()
                .run(&mono.pool)
                .await
                .context("Migration run failed")?;

            let service = Data::new(Service::new(&mono));

            let mono = Data::new(mono);
            let data = mono.clone();

            let server = HttpServer::new(move || {
                App::new()
                    .app_data(data.clone())
                    .app_data(service.clone())
                    .configure(web::configure(data.clone()))
            });

            match mono.config.server_tls.clone() {
                Some(tls_config) => {
                    crypto::aws_lc_rs::default_provider()
                        .install_default()
                        .unwrap();

                    let mut certs_file = BufReader::new(File::open(tls_config.cert_file_path)?);
                    let mut key_file = BufReader::new(File::open(tls_config.key_file_path)?);

                    let tls_certs =
                        rustls_pemfile::certs(&mut certs_file).collect::<Result<Vec<_>, _>>()?;

                    let tls_key = rustls_pemfile::pkcs8_private_keys(&mut key_file)
                        .next()
                        .unwrap()
                        .unwrap();

                    // set up TLS config options
                    let config = rustls::ServerConfig::builder()
                        .with_no_client_auth()
                        .with_single_cert(
                            tls_certs,
                            rustls::pki_types::PrivateKeyDer::Pkcs8(tls_key),
                        )
                        .unwrap();

                    server
                        .bind_rustls_0_23(&mono.config.server_addr, config)?
                        .run()
                        .await?;
                }
                None => {
                    server.bind(&mono.config.server_addr)?.run().await?;
                }
            }

            Ok(())
        }
    }
}
