use actix_web::web::{ServiceConfig, get, post};

mod handlers;
pub mod middlewares;

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.route(
        "/assets/dynamic/files/{name}",
        get().to(handlers::index::get_file),
    );

    cfg.route(
        "/assets/landing/style.css",
        get().to(handlers::index::get_bundled_css),
    );

    cfg.route(
        "/assets/dynamic/favicon.ico",
        get().to(handlers::index::get_favicon),
    );

    cfg.route(
        "/assets/landing/script.js",
        get().to(handlers::index::get_bundled_js),
    );

    cfg.route(
        "/__actions/{codename}",
        post().to(handlers::index::call_action),
    );

    cfg.route("/{path:.*}", get().to(handlers::index::get_index));
}
