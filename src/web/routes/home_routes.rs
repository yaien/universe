use actix_web::middleware::from_fn;
use actix_web::web::{ServiceConfig, scope};

use crate::web::{handlers, middlewares};

pub fn configure(config: &mut ServiceConfig) {
    config
        .service(
            scope("/dashboard")
                .service(handlers::home::index)
                .wrap(from_fn(middlewares::role)),
        )
        .service(handlers::home::assets);
}
