use minijinja::Value;
use serde::{Deserialize, Serialize};
use sqlx::prelude::Type;
use strum_macros::{Display, EnumIter};

use crate::app::auth::{Organization, User};
use crate::app::{App, AppError};

#[derive(EnumIter, Display, Debug, Serialize, Deserialize, Type, PartialEq)]
pub enum Query {
    GetProducts,
}

impl Query {
    pub async fn call(
        &self,
        app: &App,
        org: &Organization,
        user: &Option<User>,
        args: &[String],
    ) -> Result<Value, AppError> {
        match self {
            Query::GetProducts => get_products(app, org, args).await,
        }
    }

    pub fn required_args(&self) -> &[&str] {
        match self {
            Query::GetProducts => &[],
        }
    }
}

async fn get_products(app: &App, org: &Organization, args: &[String]) -> Result<Value, AppError> {
    let products = app.store.products.get_by_organization_id(&org.id).await?;
    Ok(Value::from_serialize(products))
}
