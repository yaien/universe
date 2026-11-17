use actix_web::web::{Data, ReqData};
use actix_web::{HttpRequest, get};
use maud::Markup;

use crate::app::App;
use crate::app::auth::{Organization, Role};
use crate::web::dashboard::modules::base::{Content, page};
use crate::web::errors::WebResult;

use super::roles_views as views;

#[get("/roles")]
pub async fn get_roles(
    app: Data<App>,
    org: ReqData<Organization>,
    current_role: ReqData<Role>,
    req: HttpRequest,
) -> WebResult<Markup> {
    let roles = app.auth.roles.get_by_org_id(&org.id).await?;
    Ok(page(&Content {
        title: "Roles",
        path: req.path(),
        org: &org,
        role: &current_role,
        content: views::role_list(&roles, &current_role),
    }))
}
