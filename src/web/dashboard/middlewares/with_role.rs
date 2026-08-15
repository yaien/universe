use std::ops::Deref;

use crate::app::Organization;
use crate::app::{App, User};
use crate::web::errors::WebError;

use actix_web::HttpMessage;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::error::Error;
use actix_web::middleware::Next;
use actix_web::web::{Data, ReqData};

pub async fn role<MB: MessageBody>(
    app: Data<App>,
    user: ReqData<Option<User>>,
    org: ReqData<Organization>,
    req: ServiceRequest,
    next: Next<MB>,
) -> Result<ServiceResponse<MB>, Error> {
    let Some(user) = user.deref() else {
        return Err(WebError::Redirect("/auth/google/login".into()))?;
    };

    if let Ok(role) = app
        .roles
        .get_one_by_org_id_and_user_id(&org.id, &user.id)
        .await
    {
        req.extensions_mut().insert(role);
        return next.call(req).await;
    }

    if let Ok(_) = app.invitations.accept(&org.id, &user.email, &user.id).await {
        if let Ok(role) = app
            .roles
            .get_one_by_org_id_and_user_id(&org.id, &user.id)
            .await
        {
            req.extensions_mut().insert(role);
            return next.call(req).await;
        }
    }

    Err(WebError::Redirect("/".into()))?
}
