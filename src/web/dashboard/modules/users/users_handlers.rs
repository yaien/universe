use actix_web::web::{Data, ReqData};
use actix_web::{HttpRequest, get};
use maud::Markup;

use crate::app::App;
use crate::app::auth::{Organization, Role};
use crate::web::dashboard::modules::base::{Content, page};
use crate::web::errors::WebResult;

use super::users_views as views;

#[get("/users")]
pub async fn get_users(
    app: Data<App>,
    org: ReqData<Organization>,
    role: ReqData<Role>,
    req: HttpRequest,
) -> WebResult<Markup> {
    let users = app.auth.users.get_by_organization_id(&org.id).await?;
    Ok(page(&Content {
        title: "Usuarios",
        path: req.path(),
        role: &role,
        org: &org,
        content: views::users_list(&users),
    }))
}
