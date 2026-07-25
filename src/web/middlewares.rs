use crate::{app::organization::get_organization_by_host, infra::Monolith};
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Redirect, Response, Result},
};
use tracing::info;

pub async fn with_organization(
    State(mono): State<Monolith>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let host = req.headers().get("host").and_then(|h| h.to_str().ok());

    let Some(host) = host else {
        info!("host not found");
        return Err(StatusCode::NOT_FOUND);
    };

    let schema = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("http");

    if let Some(host) = host.strip_prefix("www.") {
        let uri = req.uri().clone();
        let redirect = Redirect::to(&format!("{schema}://{host}{uri}"));
        return Ok(redirect.into_response());
    }

    let Ok(mut conn) = mono.pool.acquire().await else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let Ok(organization) = get_organization_by_host(&mut conn, host).await else {
        return Err(StatusCode::NOT_FOUND);
    };

    req.extensions_mut().insert(organization);

    Ok(next.run(req).await)
}
