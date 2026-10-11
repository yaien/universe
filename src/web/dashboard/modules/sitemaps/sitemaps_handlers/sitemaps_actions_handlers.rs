use crate::app::sitemaps::{ActionContext, CreateActionOptions, UpdateActionOptions};

use super::*;

#[derive(Deserialize)]
pub struct CreateActionForm {
    name: String,
    codename: String,
    tool: Tool,
    form_id: Option<Id>,
}

#[post("/sitemaps/actions")]
pub async fn create_action(
    org: ReqData<Organization>,
    app: Data<App>,
    session: Session,
    form: Form<CreateActionForm>,
) -> Result<Markup, WebError> {
    let (mut session_state, sitemap) =
        get_session_state_and_sitemap(&app, &session, &org.id).await?;

    let action = app
        .sitemaps
        .actions
        .create(CreateActionOptions {
            sitemap_id: &sitemap.id,
            name: &form.name,
            codename: &form.codename,
            tool: &form.tool,
            form_id: &form.form_id,
        })
        .await?;

    session_state.section = Section::EditAction;
    session_state.action_id = Some(action.id);
    session.insert("pages", session_state).ok();

    Ok(views::edit_action(&Some(action)))
}

#[derive(Deserialize)]
pub struct UpdateActionForm {
    name: String,
    codename: String,
    tool: Tool,
    response_body_template: String,
    form_id: Option<Id>,
}

#[put("/sitemaps/actions/{id}")]
pub async fn update_action(
    org: ReqData<Organization>,
    app: Data<App>,
    session: Session,
    action_id: Path<Id>,
    form: Form<UpdateActionForm>,
) -> Result<Markup, WebError> {
    let (_, sitemap) = get_session_state_and_sitemap(&app, &session, &org.id).await?;

    app.sitemaps
        .actions
        .update(UpdateActionOptions {
            action_id: &action_id,
            sitemap_id: &sitemap.id,
            name: &form.name,
            codename: &form.codename,
            tool: &form.tool,
            form_id: &form.form_id,
            response_body_template: &form.response_body_template,
        })
        .await?;

    Ok(toast("Acción actualizada correctamente", Variant::Primary))
}

#[post("/sitemaps/__actions/{codename}")]
pub async fn call_action(
    org: ReqData<Organization>,
    user: ReqData<Option<User>>,
    app: Data<App>,
    session: Session,
    codename: Path<String>,
    data: Form<HashMap<String, String>>,
) -> Result<HttpResponse, WebError> {
    let (_, sitemap) = get_session_state_and_sitemap(&app, &session, &org.id).await?;
    let action = app
        .sitemaps
        .actions
        .get_one_by_sitemap_id_and_codename(&sitemap.id, &codename)
        .await?;

    let response = action
        .call(ActionContext {
            app: app.into_inner(),
            org: Arc::new(org.into_inner()),
            user: Arc::new(user.into_inner()),
            session: Arc::new(session),
            inline: true,
            data: data.into_inner(),
        })
        .await?;

    Ok(response)
}

#[derive(Deserialize)]
pub struct GetFormsFieldsetQuery {
    pub tool: Tool,
    pub form_id: Option<Id>,
}

#[get("/sitemaps/actions/forms")]
pub async fn get_action_form_fieldset(
    org: ReqData<Organization>,
    app: Data<App>,
    query: Query<GetFormsFieldsetQuery>,
) -> Result<Markup, WebError> {
    if !query.tool.needs_form_associated() {
        return Ok(html! {});
    }

    let forms = app.forms.get_by_organization_id(&org.id).await?;
    Ok(views::form_fieldset(forms.as_slice(), query.form_id))
}
