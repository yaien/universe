mod assets;
mod middlewares;
pub mod modules;

use actix_web::middleware::from_fn;
use actix_web::web::{ServiceConfig, scope};

pub fn configure(config: &mut ServiceConfig) {
    config
        .service(
            scope("/dashboard")
                .configure(modules::home::configure)
                .configure(modules::sitemaps::configure)
                .configure(modules::products::configure)
                .wrap(from_fn(middlewares::role)),
        )
        .configure(modules::base::configure);
}
