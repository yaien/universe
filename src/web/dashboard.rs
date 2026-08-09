mod assets;
mod handlers;
mod middlewares;
mod views;

use actix_web::middleware::from_fn;
use actix_web::web::{ServiceConfig, get, post, scope};

pub fn configure(config: &mut ServiceConfig) {
    config
        .service(
            scope("/dashboard")
                .route("", get().to(handlers::home::home))
                .route("/pages", get().to(handlers::pages::get_index))
                .route("/pages/files", get().to(handlers::pages::get_files))
                .route("/pages/files", post().to(handlers::pages::upload_files))
                .wrap(from_fn(middlewares::role)),
        )
        .route(
            "/assets/static/dashboard/{filepath:.*}",
            get().to(handlers::assets::assets),
        );
}
