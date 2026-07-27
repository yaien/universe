use crate::{
    app::{App, User},
    infra::ID,
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

pub async fn user(
    app: Data<App>,
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

    let user = app.users.get_one_by_id(&user_id).await.ok();

    req.extensions_mut().insert(user);

    next.call(req).await
}
