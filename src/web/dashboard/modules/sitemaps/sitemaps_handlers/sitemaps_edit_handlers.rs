use super::*;

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

    app.sitemaps
        .pages
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

    app.sitemaps
        .layouts
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

    app.sitemaps
        .emails
        .update_subject(&sitemap.id, &email_id, &form.subject)
        .await
        .context("failed updating email info")?;

    let response = HttpResponse::Ok()
        .insert_header(RELOAD_HEADER)
        .body(toast("Email actualizado correctamente", Variant::Primary));

    Ok(response)
}
