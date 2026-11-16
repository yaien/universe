mod handlers;
pub mod middlewares;
use actix_web::web::{ServiceConfig, get, scope};

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/auth")
            .route("/logout", get().to(handlers::logout))
            .route("/google/login", get().to(handlers::login))
            .route("/google/callback", get().to(handlers::callback)),
    );
}
