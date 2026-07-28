use actix_web::web::{ServiceConfig, scope};

use crate::web::handlers;

pub fn configure(config: &mut ServiceConfig) {
    config.service(handlers::index::index);
}
