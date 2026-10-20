use crate::app::AppError;
use crate::app::sitemaps::contents::Query as VarQuery;
use crate::app::sitemaps::{NewLayoutVar, NewPageVar, Source as VarSource};

use super::*;

#[derive(Deserialize)]
pub struct CreateVar {
    pub name: String,
    pub query: VarQuery,
    pub source: Option<VarSource>,

    #[serde(flatten)]
    pub args: HashMap<String, String>,
}

#[post("/sitemaps/vars")]
pub async fn create_var(
    app: Data<App>,
    org: ReqData<Organization>,
    session: Session,
    form: Form<CreateVar>,
) -> WebResult<Markup> {
    let (mut session_state, sitemap) =
        get_session_state_and_sitemap(&app, &session, &org.id).await?;

    let Some(model_id) = session_state.model_id else {
        return Err(AppError::from("modelo invalido"))?;
    };

    let form = form.into_inner();

    match session_state.model_type {
        ModelType::Page => {
            app.sitemaps
                .vars
                .create_for_page(NewPageVar {
                    name: form.name,
                    source: form.source,
                    query: form.query,
                    args: form.args,
                    page_id: model_id,
                })
                .await?
        }
        ModelType::Layout => {
            app.sitemaps
                .vars
                .create_for_layout(NewLayoutVar {
                    name: form.name,
                    source: form.source,
                    query: form.query,
                    args: form.args,
                    layout_id: model_id,
                })
                .await?
        }
        _ => return Err(AppError::from("tipo de modelo invalido"))?,
    };

    session_state.section = Section::Vars;
    session.insert("pages", &session_state).ok();

    let query = QueryState::default();
    let state = get_view_state(&app, &org, &session, query, session_state).await?;

    Ok(html! {
        (views::vars(&state))
        (toast("Variable creada correctamente", Variant::Primary))
    })
}
