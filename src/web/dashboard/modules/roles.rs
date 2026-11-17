mod roles_handlers;
mod roles_views;

use actix_web::web::ServiceConfig;
use roles_handlers::*;

pub fn configure(config: &mut ServiceConfig) {
    config.service(get_roles);
}
