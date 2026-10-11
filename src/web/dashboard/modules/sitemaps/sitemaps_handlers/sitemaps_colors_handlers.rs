use super::*;

#[post("/sitemaps/colors")]
pub async fn create_color(
    org: ReqData<Organization>,
    app: Data<App>,
    session: Session,
) -> Result<HttpResponse, WebError> {
    let (_, sitemap) = get_session_state_and_sitemap(&app, &session, &org.id).await?;

    let color = app
        .sitemaps
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

    app.sitemaps
        .colors
        .update(&sitemap.id, &color_id, &form.tag, &form.value)
        .await
        .context("failed updating color")?;

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

    app.sitemaps
        .colors
        .delete(&sitemap.id, &color_id)
        .await
        .context("failed deleting color")?;

    let response = HttpResponse::Ok()
        .insert_header(views::RELOAD_HEADER)
        .finish();

    Ok(response)
}
