mod assets;
mod handlers;
mod middlewares;
mod views;

use actix_web::middleware::from_fn;
use actix_web::web::{ServiceConfig, get, scope};

pub fn configure(config: &mut ServiceConfig) {
    config
        .service(
            scope("/dashboard")
                .route("", get().to(handlers::home::home))
                .wrap(from_fn(middlewares::role)),
        )
        .route(
            "/assets/static/dashboard/{filepath:.*}",
            get().to(handlers::assets::assets),
        );
}
