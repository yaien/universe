use actix_web::web::ServiceConfig;

mod orders_handlers;
mod orders_views;

use orders_handlers::*;

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(order_list).service(order_details);
}
