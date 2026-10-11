use crate::web::errors::WebResult;

use super::*;

#[derive(Deserialize)]
pub struct CreateForm {
    pub name: String,
}

#[post("/sitemaps/forms")]
pub async fn create_form(
    app: Data<App>,
    org: ReqData<Organization>,
    form: Form<CreateForm>,
    session: Session,
) -> WebResult<Markup> {
    let form = app.forms.create(&org.id, &form.name).await?;

    let mut session_state = get_session_state(&session);

    session_state.form_id = Some(form.id);
    session_state.section = Section::EditForm;
    session.insert("pages", &session_state).ok();

    let form = app
        .forms
        .get_one_by_organization_id(&org.id, &form.id)
        .await?;

    Ok(views::edit_form(&Some(form)))
}
