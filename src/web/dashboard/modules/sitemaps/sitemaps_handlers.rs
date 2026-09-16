use std::ops::Deref;
use std::sync::Arc;

use actix_multipart::form::MultipartForm;
use actix_multipart::form::tempfile::TempFile;
use actix_session::Session;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Form, Path, Query, ReqData};
use actix_web::{HttpRequest, HttpResponse, delete, get, patch, post, put};
use anyhow::Context;
use maud::{Markup, html};
use minijinja::context;
use serde::Deserialize;

use crate::app::{
    App, AppError, Branch, Organization, PageInfo, RegistryContext, RenderLayoutOptions,
    RenderPageInlineOptions, Role, Scope, Sitemap, User, render_email, render_layout,
    render_page_inline,
};
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
            ModelType::Page => app
                .pages
                .get_oldest(&sitemap.id)
                .await
                .ok()
                .inspect(|page| session_state.model_id = Some(page.id))
                .map(|page| Model::Page(page)),
            ModelType::Layout => app
                .layouts
                .get_oldest(&sitemap.id)
                .await
                .ok()
                .inspect(|layout| session_state.model_id = Some(layout.id))
                .map(|layout| Model::Layout(layout)),
            ModelType::Email => app
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
    };

    match &view_state.section {
        Section::Initial => {
            view_state.sitemaps = app
                .sitemaps
                .get_drafts_by_organization_id(&org.id)
                .await
                .ok();

            view_state.pages = app
                .pages
                .get_by_sitemap_id(&view_state.sitemap.id)
                .await
                .ok();

            view_state.emails = app
                .emails
                .get_by_sitemap_id(&view_state.sitemap.id)
                .await
                .ok();

            view_state.layouts = app
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
                .pages
                .get_by_sitemap_id(&view_state.sitemap.id)
                .await
                .ok();

            view_state.layouts = app
                .layouts
                .get_by_sitemap_id(&view_state.sitemap.id)
                .await
                .ok();
        }
        Section::Edit => {
            view_state.layouts = app
                .layouts
                .get_by_sitemap_id(&view_state.sitemap.id)
                .await
                .ok();
        }
        Section::Files => {
            view_state.files = app
                .files
                .get_by_organization_id(&org.id, Scope::PAGES)
                .await
                .ok();
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
            let page = app.pages.get_by_id(&sitemap.id, &model_id).await?;

            let fonts = app.fonts.get_by_sitemap_id(&sitemap.id).await?;

            let colors = app
                .colors
                .get_by_sitemap_id(&sitemap.id)
                .await
                .map_err(|e| AppError::from(e))?;

            let layout = match page.layout_id {
                Some(layout_id) => {
                    let layout = app.layouts.get_by_id(&sitemap.id, &layout_id).await?;

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
            let layout = app.layouts.get_by_id(&sitemap.id, &model_id).await?;

            let fonts = app
                .fonts
                .get_by_sitemap_id(&sitemap.id)
                .await
                .context("failed getting fonts")?;

            let colors = app.colors.get_by_sitemap_id(&sitemap.id).await?;

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
            let email = app.emails.get_by_id(&sitemap.id, &model_id).await?;
            let ctx = context! {
                org => org.deref()
            };

            let content = render_email(&email, ctx)?;

            Ok(content)
        }
    }
}

#[derive(Debug, MultipartForm)]
pub struct UploadFilesForm {
    files: Vec<TempFile>,
}

#[post("/sitemaps/files")]
pub async fn upload_file(
    org: ReqData<Organization>,
    app: Data<App>,
    session: Session,
    MultipartForm(form): MultipartForm<UploadFilesForm>,
) -> Result<Markup, WebError> {
    app.files
        .upload_many(&org.id, form.files, Scope::PAGES)
        .await
        .context("failed uploading files")?;

    let query = QueryState::default();

    let session_state = session
        .get::<SessionState>("pages")
        .ok()
        .flatten()
        .unwrap_or_default();

    let view_state = get_view_state(&app, &org, &session, query, session_state).await?;

    Ok(views::file_grid(&view_state))
}

#[derive(Deserialize)]
pub struct UpdateFileForm {
    name: String,
}

#[put("/sitemaps/files/{file_id}")]
pub async fn update_file(
    org: ReqData<Organization>,
    app: Data<App>,
    file_id: Path<Id>,
    form: Form<UpdateFileForm>,
) -> Result<HttpResponse, WebError> {
    app.files
        .update_by_organization_id_and_id(&org.id, &file_id, &form.name)
        .await
        .context("failed saving file")?;

    let response = HttpResponse::Ok()
        .insert_header(views::RELOAD_HEADER)
        .body(html! {
            (toast("Archivo guardado correctamente", Variant::Primary))
        });

    Ok(response)
}

#[delete("/sitemaps/files/{file_id}")]
pub async fn delete_file(
    org: ReqData<Organization>,
    app: Data<App>,
    session: Session,
    file_id: Path<Id>,
) -> Result<HttpResponse, WebError> {
    let mut session_state = session
        .get::<SessionState>("pages")
        .ok()
        .flatten()
        .unwrap_or_default();

    session_state.file_id = None;
    session_state.section = Section::Files;

    session.insert("pages", &session_state).ok();

    app.files
        .delete_by_organization_id_and_id(&org.id, &file_id)
        .await
        .map_err(|e| {
            WebError::Status(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed deleting file: {e}"),
            )
        })?;

    let query = QueryState::default();

    let view_state = get_view_state(&app, &org, &session, query, session_state).await?;

    let response = HttpResponse::Ok()
        .insert_header(views::RELOAD_HEADER)
        .body(html! {
            (views::files(&view_state))
            (toast("Archivo eliminado correctamente", Variant::Primary))
        });

    Ok(response)
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

#[derive(Deserialize)]
pub struct CreateFontForm {
    pub font_id: Id,
    pub tag: String,
}

#[post("/sitemaps/fonts")]
pub async fn create_font(
    org: ReqData<Organization>,
    app: Data<App>,
    session: Session,
    form: Form<CreateFontForm>,
) -> Result<HttpResponse, WebError> {
    let (session_state, sitemap) = get_session_state_and_sitemap(&app, &session, &org.id).await?;

    app.fonts
        .create_sitemap_font(&sitemap.id, &form.font_id, &form.tag)
        .await
        .context("failed creating font")?;

    clean_font_state_response(&app, &session, session_state, &sitemap.id).await
}

#[derive(Deserialize)]
pub struct UpdateFontForm {
    pub font_id: Id,
    pub tag: String,
}

#[put("/sitemaps/fonts/{sitemap_font_id}")]
pub async fn update_font(
    org: ReqData<Organization>,
    app: Data<App>,
    session: Session,
    sitemap_font_id: Path<Id>,
    form: Form<UpdateFontForm>,
) -> Result<HttpResponse, WebError> {
    let (session_state, sitemap) = get_session_state_and_sitemap(&app, &session, &org.id).await?;

    app.fonts
        .update_sitemap_font(&sitemap_font_id, &sitemap.id, &form.font_id, &form.tag)
        .await
        .context("failed updating font")?;

    clean_font_state_response(&app, &session, session_state, &sitemap.id).await
}

async fn clean_font_state_response(
    app: &App,
    session: &Session,
    session_state: SessionState,
    sitemap_id: &Id,
) -> Result<HttpResponse, WebError> {
    let associated_fonts = app
        .fonts
        .get_by_sitemap_id(sitemap_id)
        .await
        .context("failed getting sitemap fonts")?;

    let mut session_state = session_state;

    session_state.section = Section::Fonts;
    session_state.sitemap_font_id = None;

    session.insert("pages", &session_state).ok();

    let response = HttpResponse::Ok()
        .insert_header(views::RELOAD_HEADER)
        .body(views::fonts(&Some(associated_fonts)));

    Ok(response)
}

#[post("/sitemaps/colors")]
pub async fn create_color(
    org: ReqData<Organization>,
    app: Data<App>,
    session: Session,
) -> Result<HttpResponse, WebError> {
    let (_, sitemap) = get_session_state_and_sitemap(&app, &session, &org.id).await?;

    let color = app
        .colors
        .create(&sitemap.id)
        .await
        .context("failed creating color")?;

    let response = HttpResponse::Ok()
        .insert_header(views::RELOAD_HEADER)
        .body(views::color(&color));

    Ok(response)
}

#[derive(Deserialize)]
pub struct UpdateColorForm {
    tag: String,
    value: String,
}

#[put("/sitemaps/colors/{id}")]
pub async fn update_color(
    org: ReqData<Organization>,
    app: Data<App>,
    session: Session,
    color_id: Path<Id>,
    form: Form<UpdateColorForm>,
) -> Result<HttpResponse, WebError> {
    let (_, sitemap) = get_session_state_and_sitemap(&app, &session, &org.id).await?;
    app.colors
        .update(&sitemap.id, &color_id, &form.tag, &form.value)
        .await
        .map_err(|e| {
            WebError::Status(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed updating color: {e}"),
            )
        })?;

    let response = HttpResponse::Ok()
        .insert_header(views::RELOAD_HEADER)
        .finish();

    Ok(response)
}

#[delete("/sitemaps/colors/{id}")]
pub async fn delete_color(
    org: ReqData<Organization>,
    app: Data<App>,
    session: Session,
    color_id: Path<Id>,
) -> Result<HttpResponse, WebError> {
    let (_, sitemap) = get_session_state_and_sitemap(&app, &session, &org.id).await?;

    app.colors
        .delete(&sitemap.id, &color_id)
        .await
        .context("failed deleting color")?;

    let response = HttpResponse::Ok()
        .insert_header(views::RELOAD_HEADER)
        .finish();

    Ok(response)
}

#[derive(Deserialize)]
pub struct UpdateSourceForm {
    pub source: String,
}

#[patch("/sitemaps/html")]
pub async fn update_html(
    org: ReqData<Organization>,
    app: Data<App>,
    session: Session,
    form: Form<UpdateSourceForm>,
) -> Result<HttpResponse, WebError> {
    let (session_state, sitemap) = get_session_state_and_sitemap(&app, &session, &org.id).await?;

    let Some(model_id) = session_state.model_id else {
        return Err((StatusCode::BAD_REQUEST, "missing model id in session"))?;
    };

    match session_state.model_type {
        ModelType::Page => {
            app.pages
                .update_html(&sitemap.id, &model_id, &form.source)
                .await
                .context("failed updating model html")?;
        }
        ModelType::Layout => {
            app.layouts
                .update_html(&sitemap.id, &model_id, &form.source)
                .await
                .context("failed updating model html")?;
        }
        ModelType::Email => {
            app.emails
                .update_body(&sitemap.id, &model_id, &form.source)
                .await
                .context("failed updating model body")?;
        }
    };

    Ok(HttpResponse::Ok()
        .insert_header(views::RELOAD_HEADER)
        .finish())
}

#[patch("/sitemaps/css")]
pub async fn update_css(
    org: ReqData<Organization>,
    app: Data<App>,
    session: Session,
    form: Form<UpdateSourceForm>,
) -> Result<HttpResponse, WebError> {
    let (session_state, sitemap) = get_session_state_and_sitemap(&app, &session, &org.id).await?;

    let Some(model_id) = session_state.model_id else {
        return Err((StatusCode::BAD_REQUEST, "missing model id in session"))?;
    };

    match session_state.model_type {
        ModelType::Page => {
            app.pages
                .update_css(&sitemap.id, &model_id, &form.source)
                .await
                .context("failed updating css")?;
        }
        ModelType::Layout => {
            app.layouts
                .update_css(&sitemap.id, &model_id, &form.source)
                .await
                .context("failed updating css")?;
        }
        ModelType::Email => {
            return Err((StatusCode::BAD_REQUEST, "invalid model type selected"))?;
        }
    };

    let response = HttpResponse::Ok()
        .insert_header(views::RELOAD_HEADER)
        .finish();

    Ok(response)
}

#[patch("/sitemaps/js")]
pub async fn update_js(
    org: ReqData<Organization>,
    app: Data<App>,
    session: Session,
    form: Form<UpdateSourceForm>,
) -> Result<HttpResponse, WebError> {
    let (session_state, sitemap) = get_session_state_and_sitemap(&app, &session, &org.id).await?;

    let Some(model_id) = session_state.model_id else {
        return Err((StatusCode::BAD_REQUEST, "missing model id in session"))?;
    };

    match session_state.model_type {
        ModelType::Page => {
            app.pages
                .update_js(&sitemap.id, &model_id, &form.source)
                .await
                .context("failed updating js")?;
        }
        ModelType::Layout => {
            app.layouts
                .update_js(&sitemap.id, &model_id, &form.source)
                .await
                .context("failed updating js")?;
        }
        ModelType::Email => {
            return Err(WebError::Status(
                StatusCode::BAD_REQUEST,
                "invalid model type selected".into(),
            ))?;
        }
    };

    let response = HttpResponse::Ok()
        .insert_header(views::RELOAD_HEADER)
        .finish();

    Ok(response)
}

#[derive(Deserialize)]
pub struct UpdateOgImageForm {
    file_id: Id,
}

#[patch("/sitemaps/og_image")]
pub async fn update_og_image(
    org: ReqData<Organization>,
    app: Data<App>,
    session: Session,
    form: Form<UpdateOgImageForm>,
) -> Result<Markup, WebError> {
    let (session_state, sitemap) = get_session_state_and_sitemap(&app, &session, &org.id).await?;

    if session_state.model_type != ModelType::Page {
        return Err((StatusCode::FORBIDDEN, "only pages can update og image"))?;
    }

    let Some(page_id) = session_state.model_id else {
        return Err((StatusCode::FORBIDDEN, "only pages can update og image"))?;
    };

    app.pages
        .update_og_image_file_id(&sitemap.id, &page_id, &form.file_id)
        .await
        .context("failed updating og image")?;

    Ok(views::file_og_image_active_button())
}

#[derive(Deserialize)]
pub struct UpdateFaviconForm {
    file_id: Id,
}

#[patch("/sitemaps/favicon")]
pub async fn update_favicon(
    org: ReqData<Organization>,
    app: Data<App>,
    session: Session,
    form: Form<UpdateFaviconForm>,
) -> Result<Markup, WebError> {
    let (_, sitemap) = get_session_state_and_sitemap(&app, &session, &org.id).await?;

    app.sitemaps
        .update_favicon_file_id(&sitemap.id, &form.file_id)
        .await
        .context("failed updating favicon")?;

    Ok(views::file_favicon_active_button())
}

#[derive(Deserialize)]
pub struct CreatePageForm {
    path: String,
    name: String,
    title: String,
}

#[post("/sitemaps/pages")]
pub async fn create_page(
    org: ReqData<Organization>,
    app: Data<App>,
    session: Session,
    form: Form<CreatePageForm>,
) -> Result<HttpResponse, WebError> {
    let (mut session_state, sitemap) =
        get_session_state_and_sitemap(&app, &session, &org.id).await?;

    let page = app
        .pages
        .create(&sitemap.id, &form.path, &form.name, &form.title)
        .await
        .context("failed creating page")?;

    session_state.model_id = Some(page.id.clone());
    session_state.model_type = ModelType::Page;
    session_state.section = Section::Edit;

    session.insert("pages", &session_state).ok();

    let layouts = app
        .layouts
        .get_by_sitemap_id(&sitemap.id)
        .await
        .context("failed getting layouts")?;

    let response = HttpResponse::Ok()
        .insert_header(views::RELOAD_HEADER)
        .body(views::edit(&Some(Model::Page(page)), &org, &Some(layouts)));

    Ok(response)
}

#[derive(Deserialize)]
pub struct CreateLayoutForm {
    pub name: String,
}

#[post("/sitemaps/layouts")]
pub async fn create_layout(
    org: ReqData<Organization>,
    app: Data<App>,
    session: Session,
    form: Form<CreateLayoutForm>,
) -> Result<HttpResponse, WebError> {
    let (mut session_state, sitemap) =
        get_session_state_and_sitemap(&app, &session, &org.id).await?;

    let layout = app
        .layouts
        .create(&sitemap.id, &form.name)
        .await
        .context("failed creating layout")?;

    session_state.model_id = Some(layout.id.clone());
    session_state.model_type = ModelType::Layout;
    session_state.section = Section::Edit;

    session.insert("pages", &session_state).ok();

    let response = HttpResponse::Ok()
        .insert_header(views::RELOAD_HEADER)
        .body(views::edit(&Some(Model::Layout(layout)), &org, &None));

    Ok(response)
}

#[post("/sitemaps/publish")]
pub async fn publish(
    org: ReqData<Organization>,
    app: Data<App>,
    session: Session,
) -> Result<Markup, WebError> {
    let (_, sitemap) = get_session_state_and_sitemap(&app, &session, &org.id).await?;
    app.sitemaps
        .sync_branch(&org.id, &sitemap, Branch::MAIN)
        .await
        .map_err(|e| {
            WebError::Status(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed publishing: {e}"),
            )
        })?;

    Ok(toast(
        "Mapa de sitio publicado correctamente",
        Variant::Primary,
    ))
}

#[derive(Deserialize)]
pub struct UpdatePageForm {
    name: String,
    title: String,
    path: String,
    og_description: String,
    og_type: String,
    layout_id: String,
}

#[put("/sitemaps/pages/{page_id}")]
pub async fn update_page(
    org: ReqData<Organization>,
    app: Data<App>,
    session: Session,
    page_id: Path<Id>,
    Form(form): Form<UpdatePageForm>,
) -> Result<HttpResponse, WebError> {
    let (_, sitemap) = get_session_state_and_sitemap(&app, &session, &org.id).await?;

    let layout_id: Option<Id> = form.layout_id.parse().ok();

    app.pages
        .update_info(&PageInfo {
            sitemap_id: sitemap.id.clone(),
            page_id: page_id.into_inner(),
            name: form.name,
            title: form.title,
            path: form.path,
            og_description: form.og_description,
            og_type: form.og_type,
            layout_id: layout_id,
        })
        .await
        .context("failed updating page")?;

    let response = HttpResponse::Ok()
        .insert_header(RELOAD_HEADER)
        .body(toast("Pagina actualizada correctamente", Variant::Primary));

    Ok(response)
}

#[derive(Deserialize)]
pub struct UpdateLayoutForm {
    name: String,
}

#[put("/sitemaps/layouts/{layout_id}")]
pub async fn update_layout(
    org: ReqData<Organization>,
    app: Data<App>,
    session: Session,
    layout_id: Path<Id>,
    Form(form): Form<UpdateLayoutForm>,
) -> Result<HttpResponse, WebError> {
    let (_, sitemap) = get_session_state_and_sitemap(&app, &session, &org.id).await?;

    app.layouts
        .update_name(&sitemap.id, &layout_id, &form.name)
        .await
        .context("failed updating layout info")?;

    let response = HttpResponse::Ok()
        .insert_header(RELOAD_HEADER)
        .body(toast("Layout actualizado correctamente", Variant::Primary));

    Ok(response)
}

#[derive(Deserialize)]
pub struct UpdateEmailForm {
    subject: String,
}

#[put("/sitemaps/emails/{email_id}")]
pub async fn update_email(
    org: ReqData<Organization>,
    app: Data<App>,
    session: Session,
    email_id: Path<Id>,
    Form(form): Form<UpdateEmailForm>,
) -> Result<HttpResponse, WebError> {
    let (_, sitemap) = get_session_state_and_sitemap(&app, &session, &org.id).await?;

    app.emails
        .update_subject(&sitemap.id, &email_id, &form.subject)
        .await
        .context("failed updating email info")?;

    let response = HttpResponse::Ok()
        .insert_header(RELOAD_HEADER)
        .body(toast("Email actualizado correctamente", Variant::Primary));

    Ok(response)
}

#[derive(Deserialize)]
pub struct SyncDraftForm {
    pub name: String,
}

#[post("/sitemaps/branches")]
pub async fn sync_branch(
    org: ReqData<Organization>,
    app: Data<App>,
    session: Session,
    form: Form<SyncDraftForm>,
) -> Result<Markup, WebError> {
    let (mut session_state, sitemap) =
        get_session_state_and_sitemap(&app, &session, &org.id).await?;

    let branch_name = format!("draft/{}", form.name);

    app.sitemaps
        .sync_branch(&org.id, &sitemap, &branch_name)
        .await
        .context("failed syncing draft")?;

    session_state.sitemap_branch = branch_name;
    session_state.model_id = None;
    session_state.model_type = ModelType::Page;
    session_state.section = Section::Initial;

    let query = QueryState::default();

    let view_state = get_view_state(&app, &org, &session, query, session_state).await?;

    Ok(html! {
        (views::content(&view_state))
        (toast("Mapa de sitio sincronizado correctamente", Variant::Primary))
    })
}

#[delete("/sitemaps/pages/{page_id}")]
pub async fn delete_page(
    org: ReqData<Organization>,
    app: Data<App>,
    session: Session,
    page_id: Path<Id>,
) -> Result<Markup, WebError> {
    let (mut session_state, sitemap) =
        get_session_state_and_sitemap(&app, &session, &org.id).await?;

    app.pages
        .delete_one_by_sitemap_id(&sitemap.id, &page_id)
        .await?;

    session_state.model_id = None;
    session_state.model_type = ModelType::Page;
    session_state.section = Section::Initial;

    let query = QueryState::default();

    let view_state = get_view_state(&app, &org, &session, query, session_state).await?;

    Ok(html! {
        (views::content(&view_state))
        (toast("Pagina eliminada correctamente", Variant::Primary))
    })
}

#[delete("/sitemaps/layouts/{layout_id}")]
pub async fn delete_layout(
    org: ReqData<Organization>,
    app: Data<App>,
    session: Session,
    layout_id: Path<Id>,
) -> Result<Markup, WebError> {
    let (mut session_state, sitemap) =
        get_session_state_and_sitemap(&app, &session, &org.id).await?;

    app.layouts
        .delete_one_by_sitemap_id(&sitemap.id, &layout_id)
        .await?;

    session_state.model_id = None;
    session_state.model_type = ModelType::Layout;
    session_state.section = Section::Initial;

    let query = QueryState::default();

    let view_state = get_view_state(&app, &org, &session, query, session_state).await?;

    Ok(html! {
        (views::content(&view_state))
        (toast("Layout eliminado correctamente", Variant::Primary))
    })
}

#[delete("/sitemaps/current")]
pub async fn delete_sitemap(
    org: ReqData<Organization>,
    app: Data<App>,
    session: Session,
) -> Result<Markup, WebError> {
    let session_state = get_session_state(&session);

    app.sitemaps
        .delete_one_by_organization_id(&session_state.sitemap_branch, &org.id)
        .await?;

    let session_state = SessionState::default();

    let query = QueryState::default();

    let view_state = get_view_state(&app, &org, &session, query, session_state).await?;

    Ok(html! {
        (views::content(&view_state))
        (toast("Sitemap eliminado correctamente", Variant::Primary))
    })
}
