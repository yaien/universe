mod assets;
mod handlers;
mod middlewares;
mod views;

pub use views::layout::{Variant, toast};

use actix_web::middleware::from_fn;
use actix_web::web::{ServiceConfig, get, post, scope};

use handlers::*;

pub fn configure(config: &mut ServiceConfig) {
    config
        .service(
            scope("/dashboard")
                .route("", get().to(home::home))
                .route("/empty", get().to(home::empty))
                .route("/pages", get().to(pages::get_index))
                .route("/pages", post().to(pages::exec_action))
                .route("/pages/files", post().to(pages::upload_files))
                .route("/pages/preview", get().to(pages::get_preview))
                .route("/products", get().to(products::get_index))
                .route("/products", post().to(products::exec_index_actions))
                .route("/products/{id}", get().to(products::get_details))
                .route("/products/{id}", post().to(products::exec_detail_actions))
                .route(
                    "/products/{id}/presentations/{pid}/contents",
                    post().to(products::upload_content),
                )
                .wrap(from_fn(middlewares::role)),
        )
        .route(
            "/assets/static/dashboard/{filepath:.*}",
            get().to(handlers::assets::assets),
        );
}
