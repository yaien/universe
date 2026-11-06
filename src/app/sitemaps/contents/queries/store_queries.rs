use super::*;

use anyhow::Context;

pub async fn get_products(ctx: &QueryContext<'_>) -> AppResult<Value> {
    let products = ctx
        .app
        .store
        .products
        .get_by_organization_id(&ctx.org.id)
        .await?;
    Ok(Value::from_serialize(products))
}

pub async fn get_product(ctx: &QueryContext<'_>) -> AppResult<Value> {
    let slug = match ctx.inline {
        true => ctx.args.get("test_slug"),
        false => ctx.args.get("slug"),
    };

    let slug = slug.ok_or("slug is required")?;
    let product = ctx
        .app
        .store
        .products
        .get_one_published_by_slug(&ctx.org.id, &slug)
        .await
        .context(format!("failed getting product query by slug {slug:?}"))?;
    Ok(Value::from_serialize(product))
}

pub async fn get_cart(ctx: &QueryContext<'_>) -> AppResult<Value> {
    let session_cart_id_key = match ctx.inline {
        true => "test_cart_id",
        false => "cart_id",
    };

    let Some(cart_id) = ctx.session.get(session_cart_id_key).ok().flatten() else {
        let cart = ctx
            .app
            .store
            .carts
            .create(&ctx.org.id, ctx.user.as_ref().map(|u| &u.id))
            .await
            .context("failed creating cart")?;

        ctx.session.insert(session_cart_id_key, &cart.id).ok();

        return Ok(Value::from_serialize(cart));
    };

    match ctx.app.store.carts.get(&ctx.org.id, &cart_id).await? {
        Some(cart) => Ok(Value::from_serialize(cart)),
        None => {
            let cart = ctx
                .app
                .store
                .carts
                .create(&ctx.org.id, ctx.user.as_ref().map(|u| &u.id))
                .await
                .context("failed creating cart")?;

            ctx.session.insert(session_cart_id_key, &cart.id).ok();

            return Ok(Value::from_serialize(cart));
        }
    }
}
