mod handlers;
mod middlewares;
mod routes;
mod views;

use actix_session::{SessionMiddleware, storage::CookieSessionStore};
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

        config.service(
            scope("")
                .configure(routes::index::configure)
                .configure(routes::auth::configure(mono.clone()))
                .wrap(from_fn(middlewares::user))
                .wrap(session)
                .wrap(from_fn(middlewares::organization)),
        );
    }
}
