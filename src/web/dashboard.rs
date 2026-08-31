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
                .route("/empty", get().to(handlers::home::empty))
                .route("/pages", get().to(handlers::pages::get_index))
                .route("/pages", post().to(handlers::pages::exec_action))
                .route("/pages/files", post().to(handlers::pages::upload_files))
                .route("/pages/preview", get().to(handlers::pages::get_preview))
                .route("/products", get().to(handlers::products::products))
                .route("/products", post().to(handlers::products::products_actions))
                .route(
                    "/products/{id}",
                    get().to(handlers::products::product_detail),
                )
                .wrap(from_fn(middlewares::role)),
        )
        .route(
            "/assets/static/dashboard/{filepath:.*}",
            get().to(handlers::assets::assets),
        );
}
