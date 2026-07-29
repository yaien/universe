use crate::app::App;
use crate::web::errors::{RedirectError, StatusError};
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
    let Some(host) = req.uri().host() else {
        return Err(StatusError(StatusCode::BAD_REQUEST, String::from("host not found")).into());
    };

    if host.starts_with("www.") {
        let uri = req.request().full_url().to_string().replace("www.", "");

        return Err(RedirectError(uri).into());
    }

    let Ok(org) = app.organizations.get_one_by_host(&host).await else {
        return Err(StatusError(
            StatusCode::NOT_FOUND,
            String::from("organization not found"),
        )
        .into());
    };

    req.extensions_mut().insert(org);

    next.call(req).await
}
