use std::collections::HashMap;

use actix_session::Session;
use anyhow::Context;
use minijinja::{Value, context};
use serde::{Deserialize, Serialize};
use sqlx::prelude::Type;
use strum_macros::{Display, EnumIter};

use crate::app::auth::{Organization, User};
use crate::app::store::NewCartItem;
use crate::app::{App, AppError, AppResult};
use crate::infra::Id;

#[derive(EnumIter, Display, Debug, Serialize, Deserialize, Type, PartialEq)]
pub enum Tool {
    StaticHtml,
    Form,
    AddProductToCart,
    DeleteProductFromCart,
    UpdateCartItemQuantity,
}

pub struct ToolContext<'a> {
    pub app: &'a App,
    pub org: &'a Organization,
    pub user: &'a Option<User>,
    pub session: &'a Session,
    pub inline: bool,
    pub form_id: Option<Id>,
    pub data: HashMap<String, String>,
}

impl Tool {
    pub async fn call(&self, ctx: ToolContext<'_>) -> Result<Value, AppError> {
        use Tool::*;
        match self {
            StaticHtml => static_html(ctx).await,
            Form => form(ctx).await,
            AddProductToCart => add_product_to_cart(ctx).await,
            DeleteProductFromCart => delete_product_from_cart(ctx).await,
            UpdateCartItemQuantity => update_cart_item_quantity(ctx).await,
        }
    }

    pub fn label(&self) -> &'static str {
        use Tool::*;
        match self {
            StaticHtml => "Retornar un HTML estatico",
            Form => "Guardar registro de formulario",
            AddProductToCart => "Agregar producto al carrito",
            DeleteProductFromCart => "Eliminar producto del carrito",
            UpdateCartItemQuantity => "Actualizar cantidad de item en carrito",
        }
    }

    pub fn needs_form_associated(&self) -> bool {
        use Tool::*;
        match self {
            Form => true,
            _ => false,
        }
    }
}

pub async fn static_html(_: ToolContext<'_>) -> AppResult<Value> {
    Ok(context! {})
}

pub async fn form(ctx: ToolContext<'_>) -> AppResult<Value> {
    let Some(form_id) = ctx.form_id else {
        return Err("Esta accion requiere un formulario asociado")?;
    };

    ctx.app
        .forms
        .submit(&ctx.org.id, &form_id, ctx.data)
        .await?;

    Ok(context! {})
}

pub async fn add_product_to_cart(ctx: ToolContext<'_>) -> AppResult<Value> {
    let session_key = match ctx.inline {
        true => "test_cart_id",
        false => "cart_id",
    };

    let cart = match ctx.session.get(session_key).ok().flatten() {
        Some(cart_id) => ctx.app.store.carts.get(&ctx.org.id, &cart_id).await?,
        None => None,
    };

    let cart = match cart {
        Some(cart) => cart,
        None => {
            ctx.app
                .store
                .carts
                .create(&ctx.org.id, ctx.user.as_ref().map(|u| &u.id))
                .await?
        }
    };

    let product_id = ctx
        .data
        .get("product_id")
        .map(|p| p.parse::<Id>().ok())
        .flatten()
        .ok_or("falta el id de el producto")?;

    let presentation_id = ctx
        .data
        .get("presentation_id")
        .map(|p| p.parse::<Id>().ok())
        .flatten()
        .ok_or("falta el id de la presentacion")?;

    let product = ctx
        .app
        .store
        .products
        .get_one_by_organization_id_and_id(&ctx.org.id, &product_id)
        .await
        .context("no se encontro el producto")?;

    let presentation = product
        .presentations
        .iter()
        .find(|p| p.id == presentation_id)
        .ok_or("no se encontro la presentación")?;

    let quantity = ctx
        .data
        .get("quantity")
        .map(|q| q.parse::<u32>().ok())
        .flatten()
        .ok_or("falta la cantidad")?;

    let new_item_id = ctx
        .app
        .store
        .carts
        .add_item(NewCartItem {
            organization_id: &ctx.org.id,
            cart_id: &cart.id,
            product_id: &product_id,
            presentation_id: &presentation_id,
            quantity: quantity,
        })
        .await
        .context("failed creating cart item")?;

    let cart = ctx
        .app
        .store
        .carts
        .get(&ctx.org.id, &cart.id)
        .await?
        .ok_or("fallo al consultar de nuevo el carrito de compras")?;

    Ok(context! {
        cart => cart,
        item => cart.items.iter().find(|i| i.id == new_item_id),
        product => product,
        presentation => presentation,
    })
}

pub async fn delete_product_from_cart(ctx: ToolContext<'_>) -> AppResult<Value> {
    let session_key = match ctx.inline {
        true => "test_cart_id",
        false => "cart_id",
    };

    let cart = match ctx.session.get(session_key).ok().flatten() {
        Some(cart_id) => ctx.app.store.carts.get(&ctx.org.id, &cart_id).await?,
        None => None,
    };

    let cart = match cart {
        Some(cart) => cart,
        None => {
            ctx.app
                .store
                .carts
                .create(&ctx.org.id, ctx.user.as_ref().map(|u| &u.id))
                .await?
        }
    };

    let cart_item_id = ctx
        .data
        .get("cart_item_id")
        .map(|id| id.parse::<Id>().ok())
        .flatten()
        .ok_or("falta el identificador del item")?;

    ctx.app.store.carts.delete_item(cart_item_id).await?;

    Ok(context! {})
}

pub async fn update_cart_item_quantity(ctx: ToolContext<'_>) -> AppResult<Value> {
    let session_key = match ctx.inline {
        true => "test_cart_id",
        false => "cart_id",
    };

    let cart = match ctx.session.get(session_key).ok().flatten() {
        Some(cart_id) => ctx.app.store.carts.get(&ctx.org.id, &cart_id).await?,
        None => None,
    };

    let cart = match cart {
        Some(cart) => cart,
        None => {
            ctx.app
                .store
                .carts
                .create(&ctx.org.id, ctx.user.as_ref().map(|u| &u.id))
                .await?
        }
    };

    let cart_item_id = ctx
        .data
        .get("cart_item_id")
        .map(|id| id.parse::<Id>().ok())
        .flatten()
        .ok_or("falta el identificador del item")?;

    let quantity = ctx
        .data
        .get("quantity")
        .map(|q| q.parse::<i64>().ok())
        .flatten()
        .ok_or("falta la cantidad")?;

    let Some(item) = cart.items.iter().find(|i| i.id == cart_item_id) else {
        return Err("item no encontrado")?;
    };

    if quantity < 1 {
        return Err("cantidad inferior a 1")?;
    }

    if quantity > item.presentation_quantity {
        return Err("cantidad superior al disponible")?;
    }

    ctx.app
        .store
        .carts
        .update_item_quantity(cart_item_id, quantity)
        .await?;

    Ok(context! {})
}
