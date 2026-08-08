use std::ops::Deref;

use actix_session::Session;
use actix_web::Error;
use actix_web::HttpRequest;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Query, ReqData};
use maud::{Markup, html};

use crate::app::{App, Branch, Layout, Organization, Role};
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

    let model = match session_state.model_id {
        Some(id) => match session_state.model_type {
            ModelType::Page => app
                .pages
                .get_by_id(&sitemap.id, &id)
                .await
                .ok()
                .map(|page| Model::Page(page)),
            ModelType::Layout => app
                .layouts
                .get_by_id(&sitemap.id, &id)
                .await
                .ok()
                .map(|layout| Model::Layout(layout)),
            ModelType::Email => app
                .emails
                .get_by_id(&sitemap.id, &id)
                .await
                .ok()
                .map(|email| Model::Email(email)),
        },
        None => None,
    };

    session.insert("pages", &session_state).ok();

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

pub async fn index(
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
