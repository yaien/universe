use crate::app::AppResult;
use crate::app::store::Cart;
use crate::infra::{DbPool, Id};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::prelude::{FromRow, Type};
use sqlx::types::Json;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Type, Serialize)]
pub enum OrderStatus {
    Pending,
    Completed,
    Cancelled,
}

#[derive(Type, Serialize)]
pub enum OrderProvider {
    Manual,
}

#[derive(FromRow, Serialize)]
pub struct Order {
    pub id: Id,
    pub uuid: String,
    pub organization_id: Id,
    pub cart_id: Option<Id>,
    pub form_submission_id: Option<Id>,
    pub user_id: Id,
    pub amount: f64,
    pub provider: OrderProvider,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    #[sqlx(json)]
    pub metadata: HashMap<String, String>,

    #[sqlx(skip)]
    pub items: Vec<OrderItem>,
}

#[derive(FromRow, Serialize)]
pub struct OrderItem {
    pub id: Id,
    pub order_id: Id,
    pub product_id: Id,
    pub presentation_id: Id,
    pub quantity: i32,
    pub price_per_unit: f64,
    pub total_price: f64,
}

pub struct Orders {
    pool: DbPool,
}

impl Orders {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn create_from_cart(&self, new_order: NewCartOrder) -> AppResult<Id> {
        let mut tx = self.pool.begin().await?;

        let order_id = sqlx::query(r#"
            INSERT INTO orders (uuid, organization_id, cart_id, form_submission_id, user_id, amount, provider, status, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#)
        .bind(Uuid::now_v7().to_string())
        .bind(new_order.organization_id)
        .bind(new_order.cart.id)
        .bind(new_order.form_submission_id)
        .bind(new_order.user_id)
        .bind(new_order.cart.total_price)
        .bind(new_order.provider)
        .bind(OrderStatus::Pending)
        .bind(Json(new_order.metadata))
        .execute(&mut *tx)
        .await
        .map(|r| r.last_insert_rowid())?;

        for item in &new_order.cart.items {
            if item.quantity > item.presentation_quantity {
                return Err(format!(
                    "la cantidad solicitida de {} excede la cantidad disponible en la presentación",
                    item.product_id
                ))?;
            }

            sqlx::query(r#"
                INSERT INTO order_items (order_id, product_id, presentation_id, quantity, price_per_unit, total_price)
                VALUES ($1, $2, $3, $4, $5, $6)
            "#)
            .bind(order_id)
            .bind(item.product_id)
            .bind(item.presentation_id)
            .bind(item.quantity)
            .bind(item.presentation_price)
            .bind(item.total_price)
            .execute(&mut *tx)
            .await?;

            sqlx::query("UPDATE presentations SET quantity = quantity - $1 WHERE id = $2")
                .bind(item.quantity)
                .bind(item.presentation_id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;

        Ok(order_id)
    }

    async fn preload(&self, order: &mut Order) -> AppResult<()> {
        let items =
            sqlx::query_as::<_, OrderItem>(r#"SELECT * FROM order_items WHERE order_id = $1"#)
                .bind(order.id)
                .fetch_all(&self.pool)
                .await?;

        order.items = items;

        Ok(())
    }

    pub async fn get_by_organization_id_and_id(
        &self,
        organization_id: Id,
        id: Id,
    ) -> AppResult<Order> {
        let mut order = sqlx::query_as::<_, Order>(
            r#"
            SELECT * FROM orders WHERE organization_id = $1 AND id = $2
            "#,
        )
        .bind(organization_id)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        self.preload(&mut order).await?;

        Ok(order)
    }

    pub async fn get_by_organization_id_and_user_id_and_id(
        &self,
        organization_id: &Id,
        user_id: &Id,
        id: &Id,
    ) -> AppResult<Order> {
        let mut order = sqlx::query_as::<_, Order>(
            r#"
            SELECT * FROM orders WHERE organization_id = $1 AND user_id = $2 AND id = $3
            "#,
        )
        .bind(organization_id)
        .bind(user_id)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        self.preload(&mut order).await?;

        Ok(order)
    }

    pub async fn get_by_organization_id(&self, organization_id: Id) -> AppResult<Vec<Order>> {
        let orders = sqlx::query_as::<_, Order>(
            r#"
            SELECT * FROM orders WHERE organization_id = $1
            "#,
        )
        .bind(organization_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(orders)
    }

    pub async fn get_by_organization_id_and_user_id(
        &self,
        organization_id: Id,
        user_id: Id,
    ) -> AppResult<Vec<Order>> {
        let orders = sqlx::query_as::<_, Order>(
            r#"
            SELECT * FROM orders WHERE organization_id = $1 AND user_id = $2
            "#,
        )
        .bind(organization_id)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(orders)
    }
}

pub struct NewCartOrder {
    pub organization_id: Id,
    pub cart: Cart,
    pub form_submission_id: Option<Id>,
    pub user_id: Id,
    pub provider: OrderProvider,
    pub metadata: HashMap<String, String>,
}
