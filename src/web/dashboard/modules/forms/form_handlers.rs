use super::form_views as views;
use actix_web::web::{Data, Form, Path, Query, ReqData};
use actix_web::{HttpRequest, HttpResponse, get, post};
use maud::Markup;
use serde::Deserialize;

use crate::app::App;
use crate::app::auth::{Organization, Role};
use crate::infra::Id;
use crate::web::dashboard::modules::base::{self, Content};
use crate::web::errors::WebResult;

#[derive(Deserialize)]
pub struct GetFormsQuery {
    fragment: Option<String>,
}

#[get("/forms")]
pub async fn get_forms(
    app: Data<App>,
    org: ReqData<Organization>,
    role: ReqData<Role>,
    req: HttpRequest,
    query: Query<GetFormsQuery>,
) -> WebResult<Markup> {
    let forms = app.forms.get_by_organization_id(&org.id).await?;

    match query.fragment.as_deref() {
        Some("create") => Ok(views::create_form_modal()),
        _ => Ok(base::page(&Content {
            title: "Formularios",
            path: req.path(),
            org: &org,
            role: &role,
            content: views::forms_list(forms),
        })),
    }
}

#[derive(Deserialize)]
pub struct CreateForm {
    pub name: String,
    pub codename: String,
}

#[post("/forms")]
pub async fn create_form(
    app: Data<App>,
    org: ReqData<Organization>,
    form: Form<CreateForm>,
) -> WebResult<HttpResponse> {
    let form = app
        .forms
        .create(&org.id, &form.name, &form.codename)
        .await?;

    let response = HttpResponse::Ok()
        .insert_header(("HX-Location", format!("/dashboard/forms/{}", form.id)))
        .finish();

    Ok(response)
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
        content: views::form_detail(form),
    }))
}
