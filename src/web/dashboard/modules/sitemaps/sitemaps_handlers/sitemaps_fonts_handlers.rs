use super::*;

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

    app.sitemaps
        .fonts
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

    app.sitemaps
        .fonts
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
        .sitemaps
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
