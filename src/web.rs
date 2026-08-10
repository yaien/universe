mod auth;
mod dashboard;
mod errors;
mod public;

use actix_multipart::form::tempfile::TempFileConfig;
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::middleware::Logger;
use actix_web::{
    cookie::SameSite,
    middleware::from_fn,
    web::{Data, ServiceConfig, scope},
};

use crate::infra::Monolith;

pub fn configure(mono: Data<Monolith>) -> impl Fn(&mut ServiceConfig) {
    move |config| {
        let session = SessionMiddleware::builder(
            CookieSessionStore::default(),
            mono.config.session_key.clone(),
        )
        .cookie_secure(mono.config.session_secure)
        .cookie_http_only(true)
        .cookie_same_site(SameSite::Lax)
        .build();

        let logger = Logger::default().log_level(log::Level::Debug);

        let tempfile = TempFileConfig::default().directory(&mono.config.storage_temp_path);

        config.service(
            scope("")
                .configure(dashboard::configure)
                .configure(auth::configure)
                .configure(public::configure)
                .wrap(from_fn(auth::middlewares::with_user))
                .wrap(session)
                .wrap(from_fn(public::middlewares::with_organization))
                .wrap(logger)
                .app_data(tempfile),
        );
    }
}
