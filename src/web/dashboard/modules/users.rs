mod users_handlers;
mod users_views;

use actix_web::web::ServiceConfig;
use users_handlers::*;

pub fn configure(config: &mut ServiceConfig) {
    config.service(get_users);
}
