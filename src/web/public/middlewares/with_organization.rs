use crate::app::App;
use crate::web::errors::WebError;
use actix_web::body::MessageBody;
use actix_web::http::StatusCode;
use actix_web::{
    Error, HttpMessage,
    dev::{ServiceRequest, ServiceResponse},
    middleware::Next,
    web::Data,
};

pub async fn with_organization(
    app: Data<App>,
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let Some(host) = req.request().full_url().host().map(|h| h.to_string()) else {
        return Err(WebError::Status(
            StatusCode::BAD_REQUEST,
            format!("host not found in uri {}", req.request().full_url()),
        ))?;
    };

    if host.starts_with("www.") {
        let uri = req.request().full_url().to_string().replace("www.", "");
        return Err(WebError::Redirect(uri))?;
    }

    let Ok(org) = app.organizations.get_one_by_host(&host).await else {
        return Err(WebError::Status(
            StatusCode::NOT_FOUND,
            String::from("organization not found"),
        ))?;
    };

    req.extensions_mut().insert(org);

    next.call(req).await
}
