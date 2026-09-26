use actix_session::Session;
use minijinja::{Value, context};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter};

use crate::app::auth::{Organization, User};
use crate::app::{App, AppError};

#[derive(EnumIter, Display, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tool {
    StaticHTML,
}

pub struct ToolContext<'a> {
    pub app: &'a App,
    pub organization: &'a Organization,
    pub user: &'a Option<User>,
    pub session: &'a Session,
    pub response_body: String,
}

pub enum Response {
    Value(Value),
}

impl Tool {
    pub fn call<'a>(&self, ctx: ToolContext<'a>) -> Result<Response, AppError> {
        use Tool::*;
        match self {
            StaticHTML => static_html(ctx),
        }
    }

    pub fn test<'a>(&self, ctx: ToolContext<'a>) -> Result<Response, AppError> {
        use Tool::*;
        match self {
            StaticHTML => static_html(ctx),
        }
    }

    pub fn label(&self) -> &'static str {
        use Tool::*;
        match self {
            StaticHTML => "Retorna un HTML estatico",
        }
    }
}

pub fn static_html<'a>(ctx: ToolContext<'a>) -> Result<Response, AppError> {
    Ok(Response::Value(context! {}))
}
