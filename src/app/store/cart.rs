use serde::Serialize;
use sqlx::prelude::FromRow;

use crate::app::{AppError, AppResult};
use crate::infra::{DbPool, Id};

#[derive(FromRow, Serialize)]
pub struct Cart {
    pub id: Id,

    #[sqlx(skip)]
    pub items: Vec<CartItem>,

    #[sqlx(skip)]
    pub total_price: f64,
}

#[derive(FromRow, Serialize)]
pub struct CartItem {
    pub id: Id,
    pub quantity: i64,
    pub product_id: Id,
    pub product_name: String,
    pub presentation_id: Id,
    pub presentation_name: String,
    pub presentation_price: f64,
    pub presentation_quantity: i64,
    pub first_content_file_id: Id,

    #[sqlx(skip)]
    pub total_price: f64,
}

pub struct Carts {
    db: DbPool,
}

impl Carts {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub async fn create(&self, org_id: &Id, user_id: Option<&Id>) -> AppResult<Cart> {
        let cart = sqlx::query_as::<_, Cart>(
            "insert into carts (organization_id, user_id) values ($1, $2) returning *",
        )
        .bind(org_id)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        Ok(cart)
    }

    pub async fn get(&self, org_id: &Id, cart_id: &Id) -> AppResult<Option<Cart>> {
        let cart = sqlx::query_as::<_, Cart>(
            "select * from carts where organization_id = $1 and id = $2 and deleted_at is null",
        )
        .bind(org_id)
        .bind(cart_id)
        .fetch_optional(&self.db)
        .await?;

        let Some(mut cart) = cart else {
            return Ok(None);
        };

        cart.items = sqlx::query_as::<_, CartItem>(
            r#"
                select
                    ci.id,
                    ci.quantity,
                    ci.product_id,
                    prod.name as product_name,
                    ci.presentation_id,
                    pre.name as presentation_name,
                    pre.price as presentation_price,
                    pre.quantity as presentation_quantity,
                    con.file_id as first_content_file_id
                from cart_items ci
                join products prod on ci.product_id = prod.id
                join presentations pre on ci.presentation_id = pre.id
                join contents con on con.presentation_id = ci.presentation_id
                where cart_id = $1 and con.number = 0"#,
        )
        .bind(cart_id)
        .fetch_all(&self.db)
        .await?;

        for item in cart.items.iter_mut() {
            item.total_price = (item.quantity as f64) * item.presentation_price;
            cart.total_price += item.total_price;
        }

        Ok(Some(cart))
    }

    pub async fn add_item(&self, item: NewCartItem<'_>) -> AppResult<Id> {
        sqlx::query(
            r#"
                insert into cart_items (cart_id, product_id, presentation_id, quantity)
                values ($1, $2, $3, $4)
            "#,
        )
        .bind(item.cart_id)
        .bind(item.product_id)
        .bind(item.presentation_id)
        .bind(item.quantity)
        .execute(&self.db)
        .await
        .map_err(AppError::Sqlx)
        .map(|r| r.last_insert_rowid())
    }

    pub async fn update_item_quantity(&self, cart_item_id: i64, quantity: i64) -> AppResult<()> {
        sqlx::query(
            r#"
                update cart_items set quantity = $2 where id = $1
            "#,
        )
        .bind(cart_item_id)
        .bind(quantity)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn delete_item(&self, cart_item_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
                delete from cart_items where id = $1
            "#,
        )
        .bind(cart_item_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }
}

pub struct NewCartItem<'a> {
    pub organization_id: &'a Id,
    pub cart_id: &'a Id,
    pub product_id: &'a Id,
    pub presentation_id: &'a Id,
    pub quantity: u32,
}
