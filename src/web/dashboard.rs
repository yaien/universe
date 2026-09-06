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
                .service(products::get_products)
                .service(products::create_product)
                .service(products::get_product)
                .service(products::delete_product)
                .service(products::update_product)
                .service(products::create_presentation)
                .service(products::update_presentation)
                .service(products::delete_presentation)
                .service(products::sort_presentation)
                .service(products::upload_content)
                .service(products::delete_content)
                .service(products::sort_content)
                .wrap(from_fn(middlewares::role)),
        )
        .route(
            "/assets/static/dashboard/{filepath:.*}",
            get().to(handlers::assets::assets),
        );
}
