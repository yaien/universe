use actix_web::web::ServiceConfig;

use crate::web::handlers;

pub fn configure(config: &mut ServiceConfig) {
    config.service(handlers::home::index);
}
