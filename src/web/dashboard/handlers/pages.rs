use std::ops::Deref;

use actix_multipart::form::MultipartForm;
use actix_multipart::form::tempfile::TempFile;
use actix_session::Session;
use actix_web::Error;
use actix_web::HttpRequest;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Form, Path, Query, ReqData};
use maud::{Markup, html};
use serde::Deserialize;

use crate::app::{App, Branch, Organization, Role};
use crate::infra::Id;
use crate::web::dashboard::views;
use crate::web::dashboard::views::layout::Variant;
use crate::web::dashboard::views::pages::{
    Model, ModelType, QueryState, Section, SessionState, ViewState,
};
use crate::web::errors::WebError;

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

    let sitemap = match app
        .sitemaps
        .get_one_by_branch(&org.id, &session_state.sitemap_branch)
        .await
    {
        Ok(sitemap) => sitemap,
        Err(e) => {
            let err = WebError::Status(
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
            WebError::Status(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed loading sitemap pages: {e}"),
            )
        })?;

    let emails = app
        .emails
        .get_by_sitemap_id(&sitemap.id)
        .await
        .map_err(|e| {
            WebError::Status(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed loading sitemap emails: {e}"),
            )
        })?;

    let layouts = app
        .layouts
        .get_by_sitemap_id(&sitemap.id)
        .await
        .map_err(|e| {
            WebError::Status(
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

    let mut view_state = ViewState {
        sitemap: sitemap,
        model: model,
        model_type: session_state.model_type,
        section: session_state.section,
        pages: pages,
        layouts: layouts,
        emails: emails,
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
    };

    match &view_state.section {
        Section::Files => {
            view_state.files = app.files.get_by_organization_id(&org.id).await.ok();
        }
        Section::File => {
            if let Some(file_id) = session_state.file_id {
                view_state.file = app
                    .files
                    .get_one_by_organization_id_and_id(&org.id, &file_id)
                    .await
                    .ok();
            }
        }
        Section::Fonts => {
            view_state.sitemap_fonts = app
                .fonts
                .get_by_sitemap_id(&view_state.sitemap.id)
                .await
                .ok();
        }
        Section::BrowseFonts => {
            log::warn!(
                "Query {:?}, Limit {:?}, Offset {:?}",
                view_state.browsed_font_query,
                view_state.browsed_font_limit,
                view_state.browsed_font_offset
            );

            view_state.browsed_fonts = app
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
                    .fonts
                    .get_one(&browsed_font_id)
                    .await
                    .inspect_err(|e| log::error!("failed getting browsed font: {e}"))
                    .ok()
            }

            if let Some(sitemap_font_id) = session_state.sitemap_font_id {
                view_state.sitemap_font = app
                    .fonts
                    .get_one_sitemap_font(&view_state.sitemap.id, &sitemap_font_id)
                    .await
                    .inspect_err(|e| log::error!("failed getting sitemap font: {e}"))
                    .ok();
            }
        }
        Section::Colors => {
            view_state.colors = app
                .colors
                .get_by_sitemap_id(&view_state.sitemap.id)
                .await
                .inspect_err(|e| log::error!("failed getting colors: {e}"))
                .ok();
        }
        _ => {}
    };

    Ok(view_state)
}

pub async fn get_index(
    org: ReqData<Organization>,
    role: ReqData<Role>,
    app: Data<App>,
    query: Query<QueryState>,
    session: Session,
    req: HttpRequest,
) -> Result<Markup, Error> {
    let mut query = query.into_inner();
    if !req.headers().contains_key("hx-request") {
        query = QueryState::default();
    }

    let state = get_view_state(app.as_ref(), org.deref(), query, &session).await?;

    let target = req
        .headers()
        .get("hx-target")
        .map(|h| h.to_str().ok())
        .flatten();

    match target {
        Some("article#editor") => Ok(views::pages::editor(&state)),
        Some("div#content") => Ok(views::pages::content(&state)),
        Some("div#browsed-fonts") => Ok(views::pages::browse_fonts_list(
            &state.browsed_fonts,
            &state.browsed_font_query,
            &state.browsed_font_limit,
            &state.browsed_font_offset,
        )),
        _ => Ok(views::layout::layout(&views::layout::Content {
            title: "Pages",
            path: req.path(),
            org: &org.into_inner(),
            role: &role.into_inner(),
            content: views::pages::content(&state),
        })),
    }
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
        .map_err(|e| WebError::Status(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let files = app
        .files
        .get_by_organization_id(&org.id)
        .await
        .map_err(|e| WebError::Status(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(views::pages::file_grid(&files))
}

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ActionForm {
    CreateColor,
    UpdateColor {
        id: String,
        tag: String,
        value: String,
    },
    DeleteColor {
        id: String,
    },
    SaveFont {
        tag: String,
    },
    SaveHtml {
        source: String,
    },
    SaveCss {
        source: String,
    },
    SaveJS {
        source: String,
    },
    Publish,
}

pub async fn exec_action(
    org: ReqData<Organization>,
    app: Data<App>,
    session: Session,
    Form(form): Form<ActionForm>,
) -> Result<Markup, Error> {
    let mut session_state = session
        .get::<SessionState>("pages")
        .ok()
        .flatten()
        .unwrap_or_default();

    let sitemap = app
        .sitemaps
        .get_one_by_branch(&org.id, &session_state.sitemap_branch)
        .await
        .map_err(|e| {
            WebError::Status(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("missing sitemap branch: {e}"),
            )
        })?;

    use ActionForm::*;

    match form {
        CreateColor => {
            let color = app.colors.create(&sitemap.id).await.map_err(|e| {
                WebError::Status(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("failed creating color: {e}"),
                )
            })?;

            Ok(views::pages::color(&color))
        }
        UpdateColor { id, tag, value } => {
            let id: Id = id.parse().map_err(|e| {
                WebError::Status(StatusCode::BAD_REQUEST, format!("invalid id format: {e}"))
            })?;

            app.colors
                .update(&sitemap.id, &id, &tag, &value)
                .await
                .map_err(|e| {
                    WebError::Status(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("failed updating color: {e}"),
                    )
                })?;

            Ok(html!())
        }
        DeleteColor { id } => {
            let id: Id = id.parse().map_err(|e| {
                WebError::Status(StatusCode::BAD_REQUEST, format!("invalid id format: {e}"))
            })?;

            app.colors.delete(&sitemap.id, &id).await.map_err(|e| {
                WebError::Status(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("failed updating color: {e}"),
                )
            })?;
            Ok(html!())
        }
        SaveFont { tag } => {
            let browsed_font_id = session_state.browsed_font_id.ok_or_else(|| {
                WebError::Status(
                    StatusCode::BAD_REQUEST,
                    "missing browsed font in session".into(),
                )
            })?;

            match session_state.sitemap_font_id {
                Some(sitemap_font_id) => {
                    app.fonts
                        .update_sitemap_font(&sitemap_font_id, &sitemap.id, &browsed_font_id, &tag)
                        .await
                        .map_err(|e| {
                            WebError::Status(
                                StatusCode::BAD_REQUEST,
                                format!("failed updating font: {e}"),
                            )
                        })?;
                }
                None => {
                    app.fonts
                        .create_sitemap_font(&sitemap.id, &browsed_font_id, &tag)
                        .await
                        .map_err(|e| {
                            WebError::Status(
                                StatusCode::BAD_REQUEST,
                                format!("failed creating font: {e}"),
                            )
                        })?;
                }
            };

            let associated_fonts = app
                .fonts
                .get_by_sitemap_id(&sitemap.id)
                .await
                .map_err(|e| {
                    WebError::Status(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("failed getting sitemap fonts: {e}"),
                    )
                })?;

            session_state.section = Section::Fonts;
            session_state.sitemap_font_id = None;

            session.insert("pages", &session_state).ok();

            Ok(views::pages::fonts(&Some(associated_fonts)))
        }
        SaveHtml { source } => {
            let Some(model_id) = session_state.model_id else {
                return Err(WebError::Status(
                    StatusCode::BAD_REQUEST,
                    "missing model id in session".into(),
                )
                .into());
            };

            match session_state.model_type {
                ModelType::Page => {
                    app.pages
                        .update_html(&sitemap.id, &model_id, &source)
                        .await
                        .map_err(|e| {
                            WebError::Status(
                                StatusCode::INTERNAL_SERVER_ERROR,
                                format!("failed updating model html: {e}"),
                            )
                        })?;
                }
                ModelType::Layout => {
                    app.layouts
                        .update_html(&sitemap.id, &model_id, &source)
                        .await
                        .map_err(|e| {
                            WebError::Status(
                                StatusCode::INTERNAL_SERVER_ERROR,
                                format!("failed updating model html: {e}"),
                            )
                        })?;
                }
                ModelType::Email => {
                    app.emails
                        .update_body(&sitemap.id, &model_id, &source)
                        .await
                        .map_err(|e| {
                            WebError::Status(
                                StatusCode::INTERNAL_SERVER_ERROR,
                                format!("failed updating model body: {e}"),
                            )
                        })?;
                }
            };

            Ok(html!())
        }
        SaveCss { source } => {
            let Some(model_id) = session_state.model_id else {
                return Err(WebError::Status(
                    StatusCode::BAD_REQUEST,
                    "missing model id in session".into(),
                ))?;
            };

            match session_state.model_type {
                ModelType::Page => {
                    app.pages
                        .update_css(&sitemap.id, &model_id, &source)
                        .await
                        .map_err(|e| {
                            WebError::Status(
                                StatusCode::INTERNAL_SERVER_ERROR,
                                format!("failed updating model html: {e}"),
                            )
                        })?;
                }
                ModelType::Layout => {
                    app.layouts
                        .update_css(&sitemap.id, &model_id, &source)
                        .await
                        .map_err(|e| {
                            WebError::Status(
                                StatusCode::INTERNAL_SERVER_ERROR,
                                format!("failed updating model html: {e}"),
                            )
                        })?;
                }
                ModelType::Email => {
                    return Err(WebError::Status(
                        StatusCode::BAD_REQUEST,
                        "invalid model type selected".into(),
                    ))?;
                }
            };

            Ok(html!())
        }
        SaveJS { source } => {
            let Some(model_id) = session_state.model_id else {
                return Err(WebError::Status(
                    StatusCode::BAD_REQUEST,
                    "missing model id in session".into(),
                ))?;
            };

            match session_state.model_type {
                ModelType::Page => {
                    app.pages
                        .update_js(&sitemap.id, &model_id, &source)
                        .await
                        .map_err(|e| {
                            WebError::Status(
                                StatusCode::INTERNAL_SERVER_ERROR,
                                format!("failed updating model html: {e}"),
                            )
                        })?;
                }
                ModelType::Layout => {
                    app.layouts
                        .update_js(&sitemap.id, &model_id, &source)
                        .await
                        .map_err(|e| {
                            WebError::Status(
                                StatusCode::INTERNAL_SERVER_ERROR,
                                format!("failed updating model html: {e}"),
                            )
                        })?;
                }
                ModelType::Email => {
                    return Err(WebError::Status(
                        StatusCode::BAD_REQUEST,
                        "invalid model type selected".into(),
                    ))?;
                }
            };

            Ok(html!())
        }
        Publish => {
            app.sitemaps
                .sync_branch(&org.id, &sitemap.id, Branch::MAIN)
                .await
                .map_err(|e| {
                    WebError::Status(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("failed publishing: {e}"),
                    )
                })?;

            Ok(views::layout::toast(
                "Mapa de sitio publicado correctamente",
                Variant::Primary,
            ))
        }
    }
}
