use crate::infra::Id;
use serde::Serialize;
use sqlx::prelude::{FromRow, Type};
use std::collections::HashMap;

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
