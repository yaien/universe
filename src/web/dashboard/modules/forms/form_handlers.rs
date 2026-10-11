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
        content: views::forms_list(forms),
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
        content: views::form_detail(form),
    }))
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

    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(10);
    let search = query.search.as_deref().unwrap_or("");
    let next_offset = offset + limit;

    let submissions = app
        .forms
        .submissions
        .find(&form.id, &search, &offset, &limit)
        .await?;

    Ok(views::form_submission(
        &form,
        &submissions,
        &search,
        &next_offset,
        &limit,
    ))
}
