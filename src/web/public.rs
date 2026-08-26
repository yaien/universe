use actix_web::web::{ServiceConfig, get};

mod handlers;
pub mod middlewares;

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.route(
        "/assets/dynamic/files/{name}",
        get().to(handlers::index::download_file),
    );

    cfg.route(
        "/assets/landing/style.css",
        get().to(handlers::index::get_bundled_css),
    );

    cfg.route(
        "/assets/landing/script.js",
        get().to(handlers::index::get_bundled_js),
    );

    cfg.route("/{path:.*}", get().to(handlers::index::get_index));
}
