use chrono::Utc;
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
            "select exists(select 1 from products where organization_id = $1 and id = $2 and deleted_at is null)",
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

    pub async fn update(&self, update: UpdatePresentationArgs<'_>) -> Result<(), AppError> {
        let exists: bool = sqlx::query_scalar(
            r#"
            select exists(select 1 from presentations
                join products on products.id = presentations.product_id
                join organizations on organizations.id = products.organization_id
                where organizations.id = $1 and products.id = $2 and presentations.id = $3
                    and presentations.deleted_at is null and products.deleted_at is null)
            "#,
        )
        .bind(update.organization_id)
        .bind(update.product_id)
        .bind(update.presentation_id)
        .fetch_one(&self.pool)
        .await?;

        if !exists {
            return Err(AppError::Message("no se encuentra la presentación".into()));
        }

        sqlx::query("update presentations set name = $1, quantity = $2, price = $3 where id = $4")
            .bind(update.name)
            .bind(update.quantity)
            .bind(update.price)
            .bind(update.presentation_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn delete(
        &self,
        organization_id: &Id,
        product_id: &Id,
        presentation_id: &Id,
    ) -> Result<(), AppError> {
        let exists: bool = sqlx::query_scalar(
            r#"
                    select exists(select 1 from presentations
                        join products on products.id = presentations.product_id
                        join organizations on organizations.id = products.organization_id
                        where organizations.id = $1 and products.id = $2 and presentations.id = $3
                            and presentations.deleted_at is null and products.deleted_at is null)
                    "#,
        )
        .bind(organization_id)
        .bind(product_id)
        .bind(presentation_id)
        .fetch_one(&self.pool)
        .await?;

        if !exists {
            return Err(AppError::Message("no se encuentra la presentación".into()));
        }

        sqlx::query("update presentations set deleted_at = $2 where id = $1")
            .bind(presentation_id)
            .bind(Utc::now())
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn sort(
        &self,
        organization_id: &Id,
        product_id: &Id,
        presentation_id: &Id,
        new_number: &i64,
    ) -> Result<(), AppError> {
        let current: Presentation = sqlx::query_as(
            r#"select presentations.* from presentations
                join products on products.id = presentations.product_id
                join organizations on organizations.id = products.organization_id
                where organizations.id = $1 and products.id = $2 and presentations.id = $3
                    and presentations.deleted_at is null and products.deleted_at is null
            "#,
        )
        .bind(organization_id)
        .bind(product_id)
        .bind(presentation_id)
        .fetch_one(&self.pool)
        .await?;

        log::warn!(
            "updating presentation {:?} with number {} to {new_number}",
            current.name,
            current.number
        );

        // set current number to the presentation that already has the new number
        sqlx::query("update presentations set number = $1 where product_id = $2 and number = $3 and deleted_at is null")
            .bind(&current.number)
            .bind(product_id)
            .bind(new_number)
            .execute(&self.pool)
            .await?;

        // set the new number to the current presentation by id
        sqlx::query("update presentations set number = $1 where id = $2")
            .bind(new_number)
            .bind(&current.id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

pub struct UpdatePresentationArgs<'a> {
    pub organization_id: &'a Id,
    pub product_id: &'a Id,
    pub presentation_id: &'a Id,
    pub name: &'a str,
    pub quantity: &'a i64,
    pub price: &'a i64,
}
