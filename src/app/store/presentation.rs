use sqlx::prelude::FromRow;

use crate::app::AppError;
use crate::infra::{DbPool, Id};

use super::Content;

#[derive(FromRow)]
pub struct Presentation {
    pub id: Id,
    pub product_id: Id,
    pub name: String,
    pub quantity: i64,
    pub price: f64,
    pub number: i64,

    #[sqlx(skip)]
    pub contents: Vec<Content>,
}

pub struct Presentations {
    pool: DbPool,
}

impl Presentations {
    pub const MAX_CONTENTS: usize = 5;

    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        organization_id: &Id,
        product_id: &Id,
    ) -> Result<Presentation, AppError> {
        // check if the product exists
        let exists: bool = sqlx::query_scalar(
            "select exists(select 1 from products where organization_id = $1 and id = $2)",
        )
        .bind(organization_id)
        .bind(product_id)
        .fetch_one(&self.pool)
        .await?;

        if !exists {
            return Err(AppError::Message("no se encuentra el producto".into()));
        }

        let count: i64 =
            sqlx::query_scalar("select max(number) from presentations where product_id = $1")
                .bind(product_id)
                .fetch_one(&self.pool)
                .await?;

        let number = count + 1;

        let presentation: Presentation = sqlx::query_as(
            "insert into presentations(number, name, product_id) values ($1, $2, $3) returning *",
        )
        .bind(number)
        .bind("Nueva")
        .bind(product_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(presentation)
    }
}
