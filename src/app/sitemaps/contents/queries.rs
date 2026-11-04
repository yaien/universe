use std::collections::HashMap;

use actix_session::Session;
use anyhow::Context;
use minijinja::Value;
use serde::{Deserialize, Serialize};
use sqlx::prelude::Type;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

use crate::app::auth::{Organization, User};
use crate::app::sitemaps::contents::Source;
use crate::app::store::Cart;
use crate::app::{App, AppError, AppResult};

#[derive(EnumIter, Display, Debug, Serialize, Deserialize, Type)]
pub enum Query {
    #[strum(to_string = "Obtener Productos")]
    GetProducts,

    #[strum(to_string = "Obtener Producto")]
    GetProduct,

    #[strum(to_string = "Obtener Carrito de Compras")]
    GetCart,
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
        user: &Option<User>,
        session: &Session,
        args: &HashMap<String, String>,
    ) -> Result<Value, AppError> {
        match self {
            Query::GetProducts => get_products(app, org).await,
            Query::GetProduct => get_product(app, org, args.get("slug")).await,
            Query::GetCart => get_cart(app, org, user, session, "cart_id").await,
        }
    }

    pub async fn call_inline(
        &self,
        app: &App,
        org: &Organization,
        user: &Option<User>,
        session: &Session,
        args: &HashMap<String, String>,
    ) -> Result<Value, AppError> {
        match self {
            Query::GetProduct => get_product(app, org, args.get("test_slug")).await,
            Query::GetCart => get_cart(app, org, user, session, "test_cart_id").await,
            _ => self.call(app, org, user, session, args).await,
        }
    }

    pub fn required_args(&self) -> &[&str] {
        match self {
            Query::GetProducts => &[],
            Query::GetProduct => &["slug"],
            Query::GetCart => &[],
        }
    }

    pub fn args_descriptors(&self) -> Vec<QueryArgs> {
        match self {
            Query::GetProducts => vec![],
            Query::GetProduct => vec![QueryArgs {
                name: "slug".to_string(),
                label: "slug".to_string(),
            }],
            Query::GetCart => vec![],
        }
    }

    pub fn descriptors() -> Vec<QueryDescriptor> {
        Query::iter().map(|q| q.descriptor()).collect()
    }

    pub fn descriptor(&self) -> QueryDescriptor {
        QueryDescriptor {
            name: format!("{:?}", self),
            label: self.to_string(),
            args: self.args_descriptors(),
        }
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
        .await
        .context(format!("failed getting product query by slug {slug:?}"))?;
    Ok(Value::from_serialize(product))
}

pub async fn get_cart(
    app: &App,
    org: &Organization,
    user: &Option<User>,
    session: &Session,
    session_cart_id_key: &str,
) -> AppResult<Value> {
    let Some(cart_id) = session.get(session_cart_id_key).ok().flatten() else {
        let cart = app
            .store
            .carts
            .create(&org.id, user.as_ref().map(|u| &u.id))
            .await
            .context("failed creating cart")?;

        session.insert(session_cart_id_key, &cart.id).ok();

        return Ok(Value::from_serialize(cart));
    };

    match app.store.carts.get(&org.id, &cart_id).await? {
        Some(cart) => Ok(Value::from_serialize(cart)),
        None => {
            let cart = app
                .store
                .carts
                .create(&org.id, user.as_ref().map(|u| &u.id))
                .await
                .context("failed creating cart")?;

            session.insert(session_cart_id_key, &cart.id).ok();

            return Ok(Value::from_serialize(cart));
        }
    }
}
