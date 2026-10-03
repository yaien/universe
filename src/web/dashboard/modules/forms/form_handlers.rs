use super::form_views as views;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Form, Path, Query, ReqData};
use actix_web::{HttpRequest, HttpResponse, delete, get, post, put};
use maud::{Markup, html};
use serde::Deserialize;

use crate::app::App;
use crate::app::auth::{Organization, Role};
use crate::infra::Id;
use crate::web::dashboard::modules::base::{self, Content, Variant, toast};
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

#[derive(Deserialize)]
pub struct GetFormQuery {
    pub fragment: Option<String>,
    pub field_id: Option<Id>,
}

#[get("/forms/{form_id}")]
pub async fn get_form(
    app: Data<App>,
    org: ReqData<Organization>,
    role: ReqData<Role>,
    form_id: Path<Id>,
    query: Query<GetFormQuery>,
    req: HttpRequest,
) -> WebResult<Markup> {
    let form = app
        .forms
        .get_one_by_organization_id(&org.id, &form_id)
        .await?;

    let content = match query.fragment.as_deref() {
        Some("fields") => views::field_list(&form),
        Some("create_field") => views::create_field_form(&form),
        Some("update_field") => {
            let Some(field_id) = query.field_id else {
                return Err((StatusCode::NOT_FOUND, "no se encontro el campo"))?;
            };

            let field = match form.fields.iter().find(|f| f.id == field_id) {
                Some(field) => field,
                None => return Err((StatusCode::NOT_FOUND, "no se encontro el campo"))?,
            };

            views::update_field_form(&form, &field)
        }
        _ => base::page(&Content {
            title: "Formularios",
            path: req.path(),
            org: &org,
            role: &role,
            content: views::form_detail(form),
        }),
    };

    Ok(content)
}

#[derive(Deserialize)]
pub struct FieldForm {
    pub name: String,
    pub label: String,
}

#[post("/forms/{form_id}/fields")]
pub async fn create_field(
    app: Data<App>,
    org: ReqData<Organization>,
    form_id: Path<Id>,
    form: Form<FieldForm>,
) -> WebResult<Markup> {
    app.forms
        .fields
        .create(&org.id, &form_id, &form.name, &form.label)
        .await?;

    let form = app
        .forms
        .get_one_by_organization_id(&org.id, &form_id)
        .await?;

    Ok(html! {
        (views::field_list(&form))
        (toast("Campo guardado correctamente", Variant::Primary))
    })
}

#[put("/forms/{form_id}/fields/{field_id}")]
pub async fn update_field(
    app: Data<App>,
    org: ReqData<Organization>,
    path: Path<(Id, Id)>,
    form: Form<FieldForm>,
) -> WebResult<Markup> {
    let (form_id, field_id) = path.into_inner();

    app.forms
        .fields
        .update(&org.id, &form_id, &field_id, &form.name, &form.label)
        .await?;

    Ok(toast("Campo guardado correctamente", Variant::Primary))
}

#[delete("/forms/{form_id}/fields/{field_id}")]
pub async fn delete_field(
    app: Data<App>,
    org: ReqData<Organization>,
    path: Path<(Id, Id)>,
) -> WebResult<Markup> {
    let (form_id, field_id) = path.into_inner();

    app.forms
        .fields
        .delete(&org.id, &form_id, &field_id)
        .await?;

    let form = app
        .forms
        .get_one_by_organization_id(&org.id, &form_id)
        .await?;

    Ok(html! {
        (views::field_list(&form))
        (toast("Campo eliminado correctamente", Variant::Primary))
    })
}

#[derive(Deserialize)]
pub struct GetFormSubmissionsQuery {
    pub search: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[get("/forms/{form_id}/submissions")]
pub async fn get_form_submissions(
    app: Data<App>,
    org: ReqData<Organization>,
    form_id: Path<Id>,
    query: Query<GetFormSubmissionsQuery>,
) -> WebResult<Markup> {
    let form = app
        .forms
        .get_one_by_organization_id(&org.id, &form_id)
        .await?;

    let query = query.into_inner();

    let submissions = app
        .forms
        .submissions
        .find(&form.id, &query.search, &query.offset, &query.limit)
        .await?;

    Ok(views::form_submission(
        &form,
        &submissions,
        &query.search,
        &query.offset,
        &query.limit,
    ))
}
