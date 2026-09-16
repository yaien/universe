mod base_handlers;
mod base_views;

use actix_web::web::ServiceConfig;
use base_handlers::*;
pub use base_views::*;

pub fn configure(config: &mut ServiceConfig) {
    config.service(assets);
}
