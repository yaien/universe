use std::collections::HashMap;
use std::sync::Arc;

use actix_session::Session;
use minijinja::{Value, context};
use serde::{Deserialize, Serialize};
use sqlx::prelude::Type;
use strum_macros::{Display, EnumIter};

use crate::app::auth::{Organization, User};
use crate::app::{App, AppError};
use crate::infra::Id;

#[derive(EnumIter, Display, Debug, Serialize, Deserialize, Type, PartialEq)]
pub enum Tool {
    StaticHtml,
    Form,
}

pub struct ToolContext {
    pub app: Arc<App>,
    pub org: Arc<Organization>,
    pub user: Arc<Option<User>>,
    pub session: Arc<Session>,
    pub inline: bool,
    pub form_id: Option<Id>,
    pub data: HashMap<String, String>,
}

impl Tool {
    pub async fn call(&self, ctx: ToolContext) -> Result<Value, AppError> {
        use Tool::*;
        match self {
            StaticHtml => static_html(ctx).await,
            Form => form(ctx).await,
        }
    }

    pub fn label(&self) -> &'static str {
        use Tool::*;
        match self {
            StaticHtml => "Retornar un HTML estatico",
            Form => "Guardar registro de formulario",
        }
    }

    pub fn needs_form_associated(&self) -> bool {
        use Tool::*;
        match self {
            StaticHtml => false,
            Form => true,
        }
    }
}

pub async fn static_html(_: ToolContext) -> Result<Value, AppError> {
    Ok(context! {})
}

pub async fn form(ctx: ToolContext) -> Result<Value, AppError> {
    let Some(form_id) = ctx.form_id else {
        return Err("Esta accion requiere un formulario asociado")?;
    };

    ctx.app
        .forms
        .submit(&ctx.org.id, &form_id, ctx.data)
        .await?;

    Ok(context! {})
}
