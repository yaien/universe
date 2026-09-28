use super::form_views as views;
use actix_web::web::{Data, Path, ReqData};
use actix_web::{HttpRequest, get};
use maud::Markup;

use crate::app::App;
use crate::app::auth::{Organization, Role};
use crate::infra::Id;
use crate::web::dashboard::modules::base::{self, Content};
use crate::web::errors::WebResult;

#[get("/forms")]
pub async fn get_forms(
    app: Data<App>,
    org: ReqData<Organization>,
    role: ReqData<Role>,
    req: HttpRequest,
) -> WebResult<Markup> {
    let forms = app.forms.get_by_organization_id(&org.id).await?;
    Ok(base::page(&Content {
        title: "Formularios",
        path: req.path(),
        org: &org,
        role: &role,
        content: views::forms(forms),
    }))
}

#[get("/forms/{form_id}")]
pub async fn get_form(
    app: Data<App>,
    org: ReqData<Organization>,
    role: ReqData<Role>,
    form_id: Path<Id>,
    req: HttpRequest,
) -> WebResult<Markup> {
    let form = app
        .forms
        .get_one_by_organization_id(&org.id, &form_id)
        .await?;

    Ok(base::page(&Content {
        title: "Formularios",
        path: req.path(),
        org: &org,
        role: &role,
        content: views::form(form),
    }))
}
