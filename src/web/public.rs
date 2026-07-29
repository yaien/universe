use actix_web::web::{ServiceConfig, get};

mod handlers;
pub mod middlewares;

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.route("/", get().to(handlers::index::index));
}
