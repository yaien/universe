mod handlers;
mod middlewares;
mod views;

use crate::{infra::Monolith, web::middlewares::with_organization};
use axum::{Router, middleware::from_fn_with_state, routing::get};

pub fn new_router(mono: Monolith) -> Router<()> {
    let router = Router::new();

    router
        .route("/", get(handlers::index))
        .route("/auth/google/login", get(handlers::login))
        .route("/auth/google/callback", get(handlers::callback))
        .route_layer(from_fn_with_state(mono.clone(), with_organization))
        .with_state(mono)
}
