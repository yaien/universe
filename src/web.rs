mod handlers;
mod middlewares;
mod views;

use crate::{
    infra::Monolith,
    web::handlers::{callback, index, login},
};
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::{
    cookie::SameSite,
    middleware::from_fn,
    web::{Data, ServiceConfig, get, scope},
};

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
                .service(index)
                .service(login)
                .route(&mono.config.google_callback_path, get().to(callback))
                .wrap(from_fn(middlewares::user))
                .wrap(session)
                .wrap(from_fn(middlewares::organization)),
        );
    }
}
