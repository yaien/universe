use std::collections::HashMap;
use std::sync::Arc;

use actix_session::Session;
use minijinja::{Value, context};
use serde::{Deserialize, Serialize};
use sqlx::prelude::Type;
use strum_macros::{Display, EnumIter};

use crate::app::auth::{Organization, User};
use crate::app::{App, AppError};

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
    pub data: HashMap<String, String>,
}

pub enum ToolOutput {
    Value(Value),
}

impl Tool {
    pub fn call(&self, ctx: ToolContext) -> Result<Value, AppError> {
        use Tool::*;
        match self {
            StaticHtml => static_html(ctx),
            Form => form(ctx),
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

    pub fn first() -> Self {
        Self::StaticHtml
    }
}

pub fn static_html(_: ToolContext) -> Result<Value, AppError> {
    Ok(context! {})
}

pub fn form(_: ToolContext) -> Result<Value, AppError> {
    Ok(context! {})
}
