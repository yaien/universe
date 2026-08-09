use std::ops::Deref;

use actix_multipart::form::MultipartForm;
use actix_multipart::form::tempfile::TempFile;
use actix_session::Session;
use actix_web::Error;
use actix_web::HttpRequest;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Query, ReqData};
use anyhow::Context;
use maud::Markup;

use crate::app::{App, Organization, Role};
use crate::web::dashboard::views;
use crate::web::dashboard::views::pages::{Model, ModelType, QueryState, ViewState};
use crate::web::errors::StatusError;

async fn get_view_state(
    app: &App,
    org: &Organization,
    query: QueryState,
    session: &Session,
) -> Result<ViewState, Error> {
    let mut session_state = session
        .get::<views::pages::SessionState>("pages")
        .ok()
        .flatten()
        .unwrap_or_default();

    if let Some(section) = query.section {
        session_state.section = section;
    }

    if let Some(model_type) = query.model_type {
        session_state.model_type = model_type;
        session_state.model_id = None;
    }

    if let Some(model_id) = query.model_id {
        session_state.model_id = Some(model_id);
    }

    let sitemap = match app
        .sitemaps
        .get_one_by_branch(&org.id, &session_state.sitemap_branch)
        .await
    {
        Ok(sitemap) => sitemap,
        Err(e) => {
            let err = StatusError(
                StatusCode::NOT_FOUND,
                format!("sitemap {} not found: {}", session_state.sitemap_branch, e),
            );
            return Err(err.into());
        }
    };

    let pages = app
        .pages
        .get_by_sitemap_id(&sitemap.id)
        .await
        .map_err(|e| {
            StatusError(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed loading sitemap pages: {e}"),
            )
        })?;

    let emails = app
        .emails
        .get_by_sitemap_id(&sitemap.id)
        .await
        .map_err(|e| {
            StatusError(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed loading sitemap emails: {e}"),
            )
        })?;

    let layouts = app
        .layouts
        .get_by_sitemap_id(&sitemap.id)
        .await
        .map_err(|e| {
            StatusError(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed loading sitemap layouts: {e}"),
            )
        })?;

    let model = match session_state.model_id {
        Some(id) => match session_state.model_type {
            ModelType::Page => app
                .pages
                .get_by_id(&sitemap.id, &id)
                .await
                .ok()
                .inspect(|page| session_state.model_id = Some(page.id))
                .map(|page| Model::Page(page)),
            ModelType::Layout => app
                .layouts
                .get_by_id(&sitemap.id, &id)
                .await
                .ok()
                .inspect(|layout| session_state.model_id = Some(layout.id))
                .map(|layout| Model::Layout(layout)),
            ModelType::Email => app
                .emails
                .get_by_id(&sitemap.id, &id)
                .await
                .ok()
                .inspect(|email| session_state.model_id = Some(email.id))
                .map(|email| Model::Email(email)),
        },
        None => match session_state.model_type {
            ModelType::Page => pages
                .get(0)
                .inspect(|page| session_state.model_id = Some(page.id))
                .map(|page| Model::Page(page.clone())),
            ModelType::Layout => layouts
                .get(0)
                .inspect(|layout| session_state.model_id = Some(layout.id))
                .map(|layout| Model::Layout(layout.clone())),
            ModelType::Email => emails
                .get(0)
                .inspect(|email| session_state.model_id = Some(email.id))
                .map(|email| Model::Email(email.clone())),
        },
    };

    session.insert("pages", &session_state).ok();

    let state = ViewState {
        sitemap: sitemap,
        model: model,
        model_type: session_state.model_type,
        section: session_state.section,
        pages: pages,
        layouts: layouts,
        emails: emails,
    };

    Ok(state)
}

pub async fn get_index(
    org: ReqData<Organization>,
    role: ReqData<Role>,
    app: Data<App>,
    query: Query<QueryState>,
    session: Session,
    req: HttpRequest,
) -> Result<Markup, Error> {
    let state = get_view_state(app.as_ref(), org.deref(), query.into_inner(), &session).await?;

    let target = req
        .headers()
        .get("hx-target")
        .map(|h| h.to_str().ok())
        .flatten();

    match target {
        Some("editor") => Ok(views::pages::editor(&state)),
        Some("content") => Ok(views::pages::content(&state)),
        _ => Ok(views::layout::layout(&views::layout::Content {
            title: "Pages",
            path: req.path(),
            org: &org.into_inner(),
            role: &role.into_inner(),
            content: views::pages::content(&state),
        })),
    }
}

pub async fn get_files(org: ReqData<Organization>, app: Data<App>) -> Result<Markup, Error> {
    let files = app
        .files
        .get_by_organization_id(&org.id)
        .await
        .map_err(|e| StatusError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(views::pages::file_grid(files))
}

#[derive(Debug, MultipartForm)]
pub struct UploadFilesForm {
    files: Vec<TempFile>,
}

pub async fn upload_files(
    org: ReqData<Organization>,
    app: Data<App>,
    MultipartForm(form): MultipartForm<UploadFilesForm>,
) -> Result<Markup, Error> {
    app.files
        .upload_many(&org.id, form.files)
        .await
        .map_err(|e| StatusError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    get_files(org, app).await
}
