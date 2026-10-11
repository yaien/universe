mod sitemaps_actions_handlers;
mod sitemaps_colors_handlers;
mod sitemaps_edit_handlers;
mod sitemaps_files_handlers;
mod sitemaps_fonts_handlers;
mod sitemaps_initial_handlers;
mod sitemaps_sources_handlers;

pub use sitemaps_actions_handlers::*;
pub use sitemaps_colors_handlers::*;
pub use sitemaps_edit_handlers::*;
pub use sitemaps_files_handlers::*;
pub use sitemaps_fonts_handlers::*;
pub use sitemaps_initial_handlers::*;
pub use sitemaps_sources_handlers::*;

use std::collections::HashMap;
use std::sync::Arc;

use actix_session::Session;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Form, Path, Query, ReqData};
use actix_web::{HttpRequest, HttpResponse, delete, get, patch, post, put};
use anyhow::Context;
use maud::{Markup, html};
use serde::Deserialize;

use crate::app::App;
use crate::app::auth::{Organization, Role, User};
use crate::app::sitemaps::contents::{
    EmailRegistryContext, RegistryContext, RenderLayoutOptions, RenderPageInlineOptions, Tool,
    render_email, render_layout, render_page_inline,
};

use crate::app::sitemaps::{Branch, PageInfo, Sitemap};
use crate::app::storage::Scope;
use crate::web::dashboard::modules::base::{Content, Variant, page, toast};

use super::sitemaps_views as views;
use super::sitemaps_views::{
    Model, ModelType, QueryState, RELOAD_HEADER, Section, SessionState, ViewState,
};
use crate::infra::Id;
use crate::web::errors::WebError;

async fn get_view_state<'a>(
    app: &App,
    org: &'a Organization,
    session: &Session,
    query: QueryState,
    session_state: SessionState,
) -> Result<ViewState<'a>, WebError> {
    let mut session_state = session_state;

    if let Some(section) = query.section {
        session_state.section = section;
    }

    if let Some(model_type) = query.model_type {
        session_state.model_type = model_type;
        session_state.model_id = None;
    }

    if query.model_id.is_some() {
        session_state.model_id = query.model_id;
    }

    if query.file_id.is_some() {
        session_state.file_id = query.file_id;
    }

    if query.browsed_font_id.is_some() {
        session_state.browsed_font_id = query.browsed_font_id
    }

    if query.sitemap_font_id.is_some() {
        session_state.sitemap_font_id = query.sitemap_font_id
    }

    if query.action_id.is_some() {
        session_state.action_id = query.action_id
    }

    if let Some(sitemap_branch) = query.sitemap_branch {
        session_state.sitemap_branch = sitemap_branch;
        session_state.model_id = None;
        session_state.model_type = ModelType::Page;
        session_state.section = Section::Initial;
    }

    let sitemap = match app
        .sitemaps
        .get_one_by_branch(&org.id, &session_state.sitemap_branch)
        .await
    {
        Ok(sitemap) => sitemap,
        Err(e) => {
            let message = format!("sitemap {} not found: {}", session_state.sitemap_branch, e);
            return Err((StatusCode::NOT_FOUND, message))?;
        }
    };

    let model = match session_state.model_id {
        Some(id) => match session_state.model_type {
            ModelType::Page => app
                .sitemaps
                .pages
                .get_by_id(&sitemap.id, &id)
                .await
                .ok()
                .inspect(|page| session_state.model_id = Some(page.id))
                .map(|page| Model::Page(page)),
            ModelType::Layout => app
                .sitemaps
                .layouts
                .get_by_id(&sitemap.id, &id)
                .await
                .ok()
                .inspect(|layout| session_state.model_id = Some(layout.id))
                .map(|layout| Model::Layout(layout)),
            ModelType::Email => app
                .sitemaps
                .emails
                .get_by_id(&sitemap.id, &id)
                .await
                .ok()
                .inspect(|email| session_state.model_id = Some(email.id))
                .map(|email| Model::Email(email)),
        },
        None => match session_state.model_type {
            ModelType::Page => app
                .sitemaps
                .pages
                .get_oldest(&sitemap.id)
                .await
                .ok()
                .inspect(|page| session_state.model_id = Some(page.id))
                .map(|page| Model::Page(page)),
            ModelType::Layout => app
                .sitemaps
                .layouts
                .get_oldest(&sitemap.id)
                .await
                .ok()
                .inspect(|layout| session_state.model_id = Some(layout.id))
                .map(|layout| Model::Layout(layout)),
            ModelType::Email => app
                .sitemaps
                .emails
                .get_oldest(&sitemap.id)
                .await
                .ok()
                .inspect(|email| session_state.model_id = Some(email.id))
                .map(|email| Model::Email(email)),
        },
    };

    session.insert("pages", &session_state).ok();

    let mut view_state = ViewState {
        organization: org,
        sitemap: sitemap,
        model: model,
        model_type: session_state.model_type,
        section: session_state.section,
        pages: None,
        layouts: None,
        emails: None,
        sitemaps: None,
        files: None,
        file: None,
        sitemap_fonts: None,
        sitemap_font: None,
        browsed_fonts: None,
        browsed_font: None,
        browsed_font_offset: query.browsed_fonts_offset,
        browsed_font_limit: query.browsed_fonts_limit,
        browsed_font_query: query.browsed_fonts_query,
        colors: None,
        actions: None,
        action: None,
        forms: None,
        form: None,
    };

    match &view_state.section {
        Section::Initial => {
            view_state.sitemaps = app
                .sitemaps
                .get_drafts_by_organization_id(&org.id)
                .await
                .ok();

            view_state.pages = app
                .sitemaps
                .pages
                .get_by_sitemap_id(&view_state.sitemap.id)
                .await
                .ok();

            view_state.emails = app
                .sitemaps
                .emails
                .get_by_sitemap_id(&view_state.sitemap.id)
                .await
                .ok();

            view_state.layouts = app
                .sitemaps
                .layouts
                .get_by_sitemap_id(&view_state.sitemap.id)
                .await
                .ok();
        }
        Section::Create => {
            view_state.sitemaps = app
                .sitemaps
                .get_drafts_by_organization_id(&org.id)
                .await
                .ok();
        }
        Section::Delete => {
            view_state.sitemaps = app
                .sitemaps
                .get_drafts_by_organization_id(&org.id)
                .await
                .ok();

            view_state.pages = app
                .sitemaps
                .pages
                .get_by_sitemap_id(&view_state.sitemap.id)
                .await
                .ok();

            view_state.layouts = app
                .sitemaps
                .layouts
                .get_by_sitemap_id(&view_state.sitemap.id)
                .await
                .ok();
        }
        Section::Edit => {
            view_state.layouts = app
                .sitemaps
                .layouts
                .get_by_sitemap_id(&view_state.sitemap.id)
                .await
                .ok();
        }
        Section::Files => {
            view_state.files = app
                .storage
                .get_by_organization_id(&org.id, Scope::PAGES)
                .await
                .ok();
        }
        Section::File => {
            if let Some(file_id) = session_state.file_id {
                view_state.file = app
                    .storage
                    .get_one_by_organization_id_and_id(&org.id, &file_id)
                    .await
                    .ok();
            }
        }
        Section::Fonts => {
            view_state.sitemap_fonts = app
                .sitemaps
                .fonts
                .get_by_sitemap_id(&view_state.sitemap.id)
                .await
                .ok();
        }
        Section::BrowseFonts => {
            view_state.browsed_fonts = app
                .sitemaps
                .fonts
                .find(
                    view_state.browsed_font_query.clone(),
                    view_state.browsed_font_limit.clone(),
                    view_state.browsed_font_offset.clone(),
                )
                .await
                .inspect_err(|e| log::error!("failed getting browsed fonts: {}", e))
                .ok();
        }
        Section::ConfigureFont => {
            if let Some(browsed_font_id) = session_state.browsed_font_id {
                view_state.browsed_font = app
                    .sitemaps
                    .fonts
                    .get_one(&browsed_font_id)
                    .await
                    .inspect_err(|e| log::error!("failed getting browsed font: {e}"))
                    .ok()
            }

            if let Some(sitemap_font_id) = session_state.sitemap_font_id {
                view_state.sitemap_font = app
                    .sitemaps
                    .fonts
                    .get_one_sitemap_font(&view_state.sitemap.id, &sitemap_font_id)
                    .await
                    .inspect_err(|e| log::error!("failed getting sitemap font: {e}"))
                    .ok();
            }
        }
        Section::Colors => {
            view_state.colors = app
                .sitemaps
                .colors
                .get_by_sitemap_id(&view_state.sitemap.id)
                .await
                .inspect_err(|e| log::error!("failed getting colors: {e}"))
                .ok();
        }
        Section::Actions => {
            view_state.actions = app
                .sitemaps
                .actions
                .get_by_sitemap_id(&view_state.sitemap.id)
                .await
                .inspect_err(|e| log::error!("failed getting actions: {e}"))
                .ok();
        }
        Section::EditAction => {
            if let Some(action_id) = session_state.action_id {
                view_state.action = app
                    .sitemaps
                    .actions
                    .get_one_by_sitemap_id(&view_state.sitemap.id, &action_id)
                    .await
                    .inspect_err(|e| log::error!("failed getting action: {e}"))
                    .ok();
            }
        }
        Section::CreateAction => {}
        _ => {}
    };

    Ok(view_state)
}

#[get("/sitemaps")]
pub async fn pages(
    org: ReqData<Organization>,
    role: ReqData<Role>,
    app: Data<App>,
    query: Query<QueryState>,
    session: Session,
    req: HttpRequest,
) -> Result<Markup, WebError> {
    let mut query = query.into_inner();

    if !req.headers().contains_key("hx-request") {
        query = QueryState::default();
    }

    let session_state = get_session_state(&session);

    let state = get_view_state(&app, &org, &session, query, session_state).await?;

    let target = req
        .headers()
        .get("hx-target")
        .map(|h| h.to_str().ok())
        .flatten();

    match target {
        Some("article#editor") => Ok(views::editor(&state)),
        Some("div#content") => Ok(views::content(&state)),
        Some("div#browsed-fonts") => Ok(views::browse_fonts_list(
            &state.browsed_fonts,
            &state.browsed_font_query,
            &state.browsed_font_limit,
            &state.browsed_font_offset,
        )),
        _ => Ok(page(&Content {
            title: "Mapas de Sitio",
            path: req.path(),
            org: &org,
            role: &role,
            content: views::content(&state),
        })),
    }
}

#[get("/sitemaps/preview")]
pub async fn get_preview(
    org: ReqData<Organization>,
    user: ReqData<Option<User>>,
    app: Data<App>,
    session: Session,
) -> Result<Markup, WebError> {
    let (session_state, sitemap) = get_session_state_and_sitemap(&app, &session, &org.id).await?;

    let Some(model_id) = session_state.model_id else {
        return Err((StatusCode::BAD_REQUEST, "model_id is not set"))?;
    };

    match session_state.model_type {
        ModelType::Page => {
            let page = app.sitemaps.pages.get_by_id(&sitemap.id, &model_id).await?;

            let fonts = app.sitemaps.fonts.get_by_sitemap_id(&sitemap.id).await?;

            let colors = app.sitemaps.colors.get_by_sitemap_id(&sitemap.id).await?;

            let layout = match page.layout_id {
                Some(layout_id) => {
                    let layout = app
                        .sitemaps
                        .layouts
                        .get_by_id(&sitemap.id, &layout_id)
                        .await?;

                    Some(layout)
                }
                None => None,
            };

            let ctx = RegistryContext {
                app: app.into_inner(),
                org: Arc::new(org.into_inner()),
                user: Arc::new(user.into_inner()),
            };

            let content = render_page_inline(RenderPageInlineOptions {
                ctx,
                page,
                layout,
                fonts,
                colors,
            })?;

            Ok(content)
        }
        ModelType::Layout => {
            let layout = app
                .sitemaps
                .layouts
                .get_by_id(&sitemap.id, &model_id)
                .await?;

            let fonts = app
                .sitemaps
                .fonts
                .get_by_sitemap_id(&sitemap.id)
                .await
                .context("failed getting fonts")?;

            let colors = app.sitemaps.colors.get_by_sitemap_id(&sitemap.id).await?;

            let ctx = RegistryContext {
                app: app.into_inner(),
                org: Arc::new(org.into_inner()),
                user: Arc::new(user.into_inner()),
            };

            let content = render_layout(RenderLayoutOptions {
                ctx,
                layout,
                fonts,
                colors,
            })?;

            Ok(content)
        }
        ModelType::Email => {
            let email = app
                .sitemaps
                .emails
                .get_by_id(&sitemap.id, &model_id)
                .await?;

            let ctx = EmailRegistryContext { org: &org };

            let content = render_email(ctx, &email)?;

            Ok(content)
        }
    }
}

async fn get_session_state_and_sitemap(
    app: &App,
    session: &Session,
    org_id: &Id,
) -> Result<(SessionState, Sitemap), WebError> {
    let session_state = get_session_state(session);

    let sitemap = app
        .sitemaps
        .get_one_by_branch(org_id, &session_state.sitemap_branch)
        .await
        .map_err(|e| {
            WebError::Status(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("missing sitemap branch: {e}"),
            )
        })?;

    Ok((session_state, sitemap))
}

fn get_session_state(session: &Session) -> SessionState {
    session
        .get::<SessionState>("pages")
        .ok()
        .flatten()
        .unwrap_or_default()
}
