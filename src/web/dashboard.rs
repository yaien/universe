mod assets;
mod handlers;
mod middlewares;
mod views;

pub use views::layout::{Variant, toast};

use actix_web::middleware::from_fn;
use actix_web::web::{ServiceConfig, get, scope};

use handlers::*;

pub fn configure(config: &mut ServiceConfig) {
    config
        .service(
            scope("/dashboard")
                .route("", get().to(home::home))
                .route("/empty", get().to(home::empty))
                .service(pages::pages)
                .service(pages::get_preview)
                .service(pages::create_page)
                .service(pages::create_layout)
                .service(pages::publish)
                .service(pages::update_page)
                .service(pages::update_layout)
                .service(pages::update_email)
                .service(pages::delete_page)
                .service(pages::delete_layout)
                .service(pages::delete_sitemap)
                .service(pages::sync_branch)
                .service(pages::upload_file)
                .service(pages::update_file)
                .service(pages::delete_file)
                .service(pages::create_font)
                .service(pages::update_font)
                .service(pages::create_color)
                .service(pages::update_color)
                .service(pages::delete_color)
                .service(pages::update_html)
                .service(pages::update_css)
                .service(pages::update_js)
                .service(pages::update_og_image)
                .service(pages::update_favicon)
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
