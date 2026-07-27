use actix_web::web::{Data, ServiceConfig, get, scope};

use crate::infra::Monolith;
use crate::web::handlers::auth::*;

pub fn configure(mono: Data<Monolith>) -> impl Fn(&mut ServiceConfig) {
    move |cfg| {
        cfg.service(
            scope("")
                .route("/auth/google/login", get().to(login))
                .route(&mono.config.google_callback_path, get().to(callback)),
        );
    }
}
