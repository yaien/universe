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

    #[strum(to_string = "Obtener Producto")]
    GetProduct,
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
    pub label: String,
}

impl Query {
    pub async fn call(
        &self,
        app: &App,
        org: &Organization,
        _user: &Option<User>,
        args: &HashMap<String, String>,
    ) -> Result<Value, AppError> {
        match self {
            Query::GetProducts => get_products(app, org).await,
            Query::GetProduct => get_product(app, org, args.get("slug")).await,
        }
    }

    pub async fn call_inline(
        &self,
        app: &App,
        org: &Organization,
        user: &Option<User>,
        args: &HashMap<String, String>,
    ) -> Result<Value, AppError> {
        match self {
            Query::GetProduct => get_product(app, org, args.get("test_slug")).await,
            _ => self.call(app, org, user, args).await,
        }
    }

    pub fn required_args(&self) -> &[&str] {
        match self {
            Query::GetProducts => &[],
            Query::GetProduct => &["slug"],
        }
    }

    pub fn args_descriptors(&self) -> Vec<QueryArgs> {
        match self {
            Query::GetProducts => vec![],
            Query::GetProduct => vec![QueryArgs {
                name: "Slug".to_string(),
                label: "slug".to_string(),
            }],
        }
    }

    pub fn descriptors() -> Vec<QueryDescriptor> {
        Query::iter()
            .map(|q| QueryDescriptor {
                name: format!("{q:?}"),
                label: q.to_string(),
                args: q.args_descriptors(),
            })
            .collect()
    }
}

async fn get_products(app: &App, org: &Organization) -> Result<Value, AppError> {
    let products = app.store.products.get_by_organization_id(&org.id).await?;
    Ok(Value::from_serialize(products))
}

async fn get_product(
    app: &App,
    org: &Organization,
    slug: Option<&String>,
) -> Result<Value, AppError> {
    let slug = slug.ok_or("slug is required")?;
    let product = app
        .store
        .products
        .get_one_published_by_slug(&org.id, &slug)
        .await?;
    Ok(Value::from_serialize(product))
}
