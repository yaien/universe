use super::*;

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
            app.sitemaps
                .pages
                .update_html(&sitemap.id, &model_id, &form.source)
                .await
                .context("failed updating model html")?;
        }
        ModelType::Layout => {
            app.sitemaps
                .layouts
                .update_html(&sitemap.id, &model_id, &form.source)
                .await
                .context("failed updating model html")?;
        }
        ModelType::Email => {
            app.sitemaps
                .emails
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
            app.sitemaps
                .pages
                .update_css(&sitemap.id, &model_id, &form.source)
                .await
                .context("failed updating css")?;
        }
        ModelType::Layout => {
            app.sitemaps
                .layouts
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
            app.sitemaps
                .pages
                .update_js(&sitemap.id, &model_id, &form.source)
                .await
                .context("failed updating js")?;
        }
        ModelType::Layout => {
            app.sitemaps
                .layouts
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
