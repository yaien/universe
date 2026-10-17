use actix_web::web::{Data, Query};
use actix_web::{HttpResponse, get};
use serde::Deserialize;

use crate::app::App;

#[derive(Deserialize)]
struct Caddy {
    domain: String,
}

#[get("/__caddy")]
pub async fn caddy(app: Data<App>, query: Query<Caddy>) -> HttpResponse {
    log::info!("caddy validation to domain: {}", query.domain);
    match app.auth.organizations.get_one_by_host(&query.domain).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}
