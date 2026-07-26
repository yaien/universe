mod handlers;
mod middlewares;
mod views;

use std::pin::Pin;

use crate::{
    infra::Monolith,
    web::handlers::{callback, index, login},
};
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::{
    middleware::from_fn,
    web::{self, Data, ServiceConfig},
};

pub fn configure(mono: Data<Monolith>) -> impl Fn(&mut ServiceConfig) {
    move |config| {
        let session = SessionMiddleware::builder(
            CookieSessionStore::default(),
            mono.config.session_key.clone(),
        )
        .build();

        config.service(
            web::scope("")
                .service(index)
                .service(login)
                .service(callback)
                .wrap(from_fn(middlewares::user))
                .wrap(session)
                .wrap(from_fn(middlewares::organization)),
        );
    }
}
