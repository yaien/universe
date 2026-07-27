use actix_web::web::{ServiceConfig, scope};

use crate::web::handlers;

pub fn configure(config: &mut ServiceConfig) {
    config.service(scope("").service(handlers::index::index));
}
