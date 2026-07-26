use crate::{
    app::{User, get_user_by_id, organization::get_organization_by_host},
    infra::{ID, Monolith},
};
use actix_session::Session;
use actix_web::{
    Error, HttpMessage, HttpResponse,
    body::{EitherBody, MessageBody},
    dev::{ServiceRequest, ServiceResponse},
    error,
    http::header,
    middleware::Next,
    web::Data,
};

pub async fn organization(
    mono: Data<Monolith>,
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<EitherBody<impl MessageBody>>, Error> {
    let Some(host) = req.uri().host() else {
        return Err(error::ErrorBadRequest("host header is missing"));
    };

    if host.starts_with("www.") {
        let uri = req.request().full_url().to_string().replace("www.", "");

        let response = HttpResponse::SeeOther()
            .insert_header((header::LOCATION, uri))
            .finish();

        let redirect = req.into_response(response).map_into_boxed_body();

        return Ok(redirect.map_into_right_body());
    }

    let Ok(mut conn) = mono.pool.acquire().await else {
        return Err(error::ErrorInternalServerError(
            "failed to acquire connection",
        ));
    };

    let Ok(org) = get_organization_by_host(&mut conn, &host).await else {
        return Err(error::ErrorNotFound("organization not found"));
    };

    req.extensions_mut().insert(org);

    let res = next.call(req).await?;
    Ok(res.map_into_left_body())
}

pub async fn user(
    mono: Data<Monolith>,
    session: Session,
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let Some(user_id) = session
        .get::<String>("user_id")
        .ok()
        .flatten()
        .and_then(|id| id.parse::<ID>().ok())
    else {
        req.extensions_mut().insert::<Option<User>>(None);
        return next.call(req).await;
    };

    let Ok(mut conn) = mono.pool.acquire().await else {
        return Err(error::ErrorInternalServerError(
            "failed to acquire connection",
        ));
    };

    let user = get_user_by_id(&mut conn, &user_id).await.ok();

    req.extensions_mut().insert(user);

    next.call(req).await
}
