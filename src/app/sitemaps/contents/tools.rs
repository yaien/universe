use std::collections::HashMap;
use std::sync::Arc;

use actix_session::Session;
use minijinja::{Value, context};
use serde::{Deserialize, Serialize};
use sqlx::prelude::Type;
use strum_macros::{Display, EnumIter};

use crate::app::auth::{Organization, User};
use crate::app::{App, AppError};

#[derive(EnumIter, Display, Debug, Serialize, Deserialize, Type)]
pub enum Tool {
    StaticHtml,
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
        }
    }

    pub fn label(&self) -> &'static str {
        use Tool::*;
        match self {
            StaticHtml => "Retorna un HTML estatico",
        }
    }
}

pub fn static_html(_: ToolContext) -> Result<Value, AppError> {
    Ok(context! {})
}
