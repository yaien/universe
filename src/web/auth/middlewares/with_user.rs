use crate::{
    app::{App, User},
    infra::Id,
};
use actix_session::Session;
use actix_web::{
    Error, HttpMessage,
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::Next,
    web::Data,
};

pub async fn with_user<MB: MessageBody>(
    app: Data<App>,
    session: Session,
    req: ServiceRequest,
    next: Next<MB>,
) -> Result<ServiceResponse<MB>, Error> {
    let Some(user_id) = session
        .get::<String>("user_id")
        .ok()
        .flatten()
        .and_then(|id| id.parse::<Id>().ok())
    else {
        req.extensions_mut().insert::<Option<User>>(None);
        return next.call(req).await;
    };

    let user = app.users.get_one_by_id(&user_id).await.ok();

    req.extensions_mut().insert(user);

    next.call(req).await
}
