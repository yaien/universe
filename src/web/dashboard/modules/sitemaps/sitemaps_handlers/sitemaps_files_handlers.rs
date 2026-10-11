use actix_multipart::form::MultipartForm;
use actix_multipart::form::tempfile::TempFile;

use super::*;

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
    app.storage
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
    app.storage
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

    app.storage
        .delete_by_organization_id_and_id(&org.id, &file_id)
        .await
        .context("failed deleting file")?;

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

    app.sitemaps
        .pages
        .update_og_image_file_id(&sitemap.id, &page_id, &form.file_id)
        .await
        .context("failed updating og image")?;

    Ok(views::file_og_image_active_button(false))
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

    Ok(views::file_favicon_active_button(false))
}
