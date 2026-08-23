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

use crate::app::FileConversion;

#[tokio::main]
async fn main() -> Result<()> {
    let cmd = Args::parse();

    let mono = Monolith::build_from_env()
        .await
        .context("failed initializing monolith")?;

    migrate!()
        .run(&mono.pool)
        .await
        .context("Migration run failed")?;

    let service = Service::new(&mono);

    match &cmd.command {
        Some(Command::Create {
            url,
            hostname,
            title,
            email,
        }) => {
            let org_id = service
                .organizations
                .create(url, hostname, title)
                .await
                .context("failed creating organization")?;

            let exp = Utc::now() + Duration::hours(3);

            service
                .invitations
                .create(&org_id, email, &exp)
                .await
                .context("failed inviting user")?;

            return Ok(());
        }
        Some(Command::Invite { email, hostname }) => {
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
            let mono = Data::new(mono);
            let service = Data::new(service);
            let data = mono.clone();

            let worker = mono.worker.clone();
            let worker_srv = service.clone();

            tokio::spawn(async move {
                let mut w = worker.lock().await;
                w.procesor(Box::new(FileConversion::new(worker_srv.files.clone())));
                w.work().await;
            });

            let fetcher = mono.fetcher.clone();
            tokio::spawn(async move {
                fetcher.start().await;
            });

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
