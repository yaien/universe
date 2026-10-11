use super::*;

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
        .sitemaps
        .pages
        .create(&sitemap.id, &form.path, &form.name, &form.title)
        .await
        .context("failed creating page")?;

    session_state.model_id = Some(page.id.clone());
    session_state.model_type = ModelType::Page;
    session_state.section = Section::Edit;

    session.insert("pages", &session_state).ok();

    let layouts = app
        .sitemaps
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
        .sitemaps
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

    app.sitemaps
        .pages
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

    app.sitemaps
        .layouts
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
