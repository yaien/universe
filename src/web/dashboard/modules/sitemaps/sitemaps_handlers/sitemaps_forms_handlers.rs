use crate::app::AppError;
use crate::web::dashboard::modules::sitemaps::sitemaps_views::create_field_form;
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

#[put("/sitemaps/forms/{form_id}")]
pub async fn update_form(
    app: Data<App>,
    org: ReqData<Organization>,
    form_id: Path<Id>,
    form: Form<CreateForm>,
) -> WebResult<Markup> {
    app.forms.update(&org.id, &form_id, &form.name).await?;
    Ok(toast(
        "Formulario actualizado correctamente",
        Variant::Primary,
    ))
}

#[get("/sitemaps/forms/{form_id}/delete")]
pub async fn get_delete_form(
    app: Data<App>,
    org: ReqData<Organization>,
    form_id: Path<Id>,
) -> WebResult<Markup> {
    let form = app
        .forms
        .get_one_by_organization_id(&org.id, &form_id)
        .await?;

    Ok(views::delete_form(&form))
}

#[delete("/sitemaps/forms/{form_id}")]
pub async fn delete_form(
    app: Data<App>,
    org: ReqData<Organization>,
    form_id: Path<Id>,
    session: Session,
) -> WebResult<Markup> {
    app.forms.delete(&org.id, &form_id).await?;

    let mut session_state = get_session_state(&session);
    session_state.section = Section::Forms;
    session_state.form_id = None;
    session.insert("pages", &session_state).ok();

    let forms = app.forms.get_by_organization_id(&org.id).await?;

    Ok(html! {
        (toast("Formulario eliminado correctamente", Variant::Primary))
        (views::forms(&Some(forms)))
    })
}

#[get("/sitemaps/forms/{form_id}/fields/create")]
pub async fn get_create_form_field_form(
    app: Data<App>,
    org: ReqData<Organization>,
    form_id: Path<Id>,
) -> WebResult<Markup> {
    let form = app
        .forms
        .get_one_by_organization_id(&org.id, &form_id)
        .await?;

    Ok(create_field_form(&form))
}

#[derive(Deserialize)]
pub struct FieldForm {
    pub name: String,
    pub label: String,
}

#[post("/sitemaps/forms/{form_id}/fields")]
pub async fn create_form_field(
    app: Data<App>,
    org: ReqData<Organization>,
    form_id: Path<Id>,
    form: Form<FieldForm>,
) -> WebResult<Markup> {
    app.forms
        .fields
        .create(&org.id, &form_id, &form.name, &form.label)
        .await?;

    let form = app
        .forms
        .get_one_by_organization_id(&org.id, &form_id)
        .await?;

    Ok(html! {
        (views::form_field_list(&form))
        (toast("Campo guardado correctamente", Variant::Primary))
    })
}

#[get("/sitemaps/forms/{form_id}/fields/{field_id}")]
pub async fn get_edit_form_field_form(
    app: Data<App>,
    org: ReqData<Organization>,
    path: (Path<(Id, Id)>),
) -> WebResult<Markup> {
    let (form_id, field_id) = path.into_inner();

    let form = app
        .forms
        .get_one_by_organization_id(&org.id, &form_id)
        .await?;

    let Some(field) = form.fields.iter().find(|field| field.id == field_id) else {
        return Err(AppError::from("no se encuentra el campo"))?;
    };

    Ok(views::update_field_form(&form, field))
}

#[put("/sitemaps/forms/{form_id}/fields/{field_id}")]
pub async fn update_form_field(
    app: Data<App>,
    org: ReqData<Organization>,
    path: Path<(Id, Id)>,
    form: Form<FieldForm>,
) -> WebResult<Markup> {
    let (form_id, field_id) = path.into_inner();

    app.forms
        .fields
        .update(&org.id, &form_id, &field_id, &form.name, &form.label)
        .await?;

    Ok(toast("Campo guardado correctamente", Variant::Primary))
}

#[delete("/sitemaps/forms/{form_id}/fields/{field_id}")]
pub async fn delete_form_field(
    app: Data<App>,
    org: ReqData<Organization>,
    path: Path<(Id, Id)>,
) -> WebResult<Markup> {
    let (form_id, field_id) = path.into_inner();

    app.forms
        .fields
        .delete(&org.id, &form_id, &field_id)
        .await?;

    let form = app
        .forms
        .get_one_by_organization_id(&org.id, &form_id)
        .await?;

    Ok(html! {
        (views::form_field_list(&form))
        (toast("Campo eliminado correctamente", Variant::Primary))
    })
}
