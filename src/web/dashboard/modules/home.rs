use actix_web::web::ServiceConfig;

mod home_handlers;

use home_handlers::*;

pub fn configure(config: &mut ServiceConfig) {
    config.service(empty).service(home);
}
