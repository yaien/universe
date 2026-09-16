mod products_handlers;
mod products_views;

use actix_web::web::ServiceConfig;
use products_handlers::*;

pub fn configure(config: &mut ServiceConfig) {
    config
        .service(get_products)
        .service(create_product)
        .service(get_product)
        .service(delete_product)
        .service(update_product)
        .service(create_presentation)
        .service(update_presentation)
        .service(delete_presentation)
        .service(sort_presentation)
        .service(upload_content)
        .service(delete_content)
        .service(sort_content);
}
