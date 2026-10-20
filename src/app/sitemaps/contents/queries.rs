use std::collections::HashMap;

use minijinja::Value;
use serde::{Deserialize, Serialize};
use sqlx::prelude::Type;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

use crate::app::auth::{Organization, User};
use crate::app::{App, AppError};

#[derive(EnumIter, Display, Debug, Serialize, Deserialize, Type)]
pub enum Query {
    #[strum(to_string = "Obtener Productos")]
    GetProducts,
}

#[derive(Serialize)]
pub struct QueryDescriptor {
    pub name: String,
    pub label: String,
    pub args: Vec<QueryArgs>,
}

#[derive(Serialize)]
pub struct QueryArgs {
    pub name: String,
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

    pub fn descriptors() -> Vec<QueryDescriptor> {
        Query::iter()
            .map(|q| QueryDescriptor {
                name: format!("{q:?}"),
                label: q.to_string(),
                args: Vec::new(),
            })
            .collect()
    }
}

async fn get_products(app: &App, org: &Organization, args: &[String]) -> Result<Value, AppError> {
    let products = app.store.products.get_by_organization_id(&org.id).await?;
    Ok(Value::from_serialize(products))
}
