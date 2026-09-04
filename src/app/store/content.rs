use std::sync::Arc;

use actix_multipart::form::tempfile::TempFile;
use sqlx::prelude::FromRow;

use crate::app::{AppError, Files};
use crate::infra::{DbPool, Id};

#[derive(FromRow)]
pub struct Content {
    pub id: Id,
    pub number: i64,
    pub presentation_id: Id,
    pub file_id: Id,
    pub file_preset: String,
}

pub const MAX_CONTENTS_PER_PRESENTATION: i64 = 5;

pub struct Contents {
    pool: DbPool,
    files: Arc<Files>,
}

impl Contents {
    pub fn new(pool: DbPool, files: Arc<Files>) -> Self {
        Self { pool, files }
    }

    pub async fn upload_many(
        &self,
        organization_id: &Id,
        product_id: &Id,
        presentation_id: &Id,
        files: Vec<TempFile>,
    ) -> Result<Vec<Id>, AppError> {
        let exists: bool = sqlx::query_scalar(
            r#"select exists(
                select * from presentations
                    join products on presentations.product_id = products.id
                    join organizations on products.organization_id = organizations.id
                    where organizations.id = $1 and products.id = $2 and presentations.id = $3
                        and presentations.deleted_at is null and products.deleted_at is null
            )"#,
        )
        .bind(organization_id)
        .bind(product_id)
        .bind(presentation_id)
        .fetch_one(&self.pool)
        .await?;

        if !exists {
            return Err("no se encuentra la presentación")?;
        }

        let count: i64 =
            sqlx::query_scalar("select count(*) from contents where presentation_id = $1")
                .bind(presentation_id)
                .fetch_one(&self.pool)
                .await?;

        if count >= MAX_CONTENTS_PER_PRESENTATION {
            return Err("se ha alcanzado el límite de contenido")?;
        }

        if count + files.len() as i64 > MAX_CONTENTS_PER_PRESENTATION {
            return Err(format!(
                "haz alcanzado el limite de contenido, solo puedes subir {} archivos mas",
                MAX_CONTENTS_PER_PRESENTATION - count
            ))?;
        }

        let files_id = self.files.upload_many(organization_id, files).await?;

        let mut content_ids = Vec::new();

        for (i, file_id) in files_id.iter().enumerate() {
            let number = count + i as i64;

            let content_id = sqlx::query(
                "insert into contents(presentation_id, file_id, number) values ($1, $2, $3)",
            )
            .bind(presentation_id)
            .bind(file_id)
            .bind(number)
            .execute(&self.pool)
            .await
            .map(|r| r.last_insert_rowid())?;

            content_ids.push(content_id);
        }

        Ok(content_ids)
    }

    pub async fn delete(
        &self,
        organization_id: &Id,
        product_id: &Id,
        presentation_id: &Id,
        content_id: &Id,
    ) -> Result<(), AppError> {
        let content = sqlx::query_as::<_, Content>(r#"
                select contents.*, files.preset as file_preset
                    from contents
                    join files on contents.file_id = files.id
                    join presentations on contents.presentation_id = presentations.id
                    join products on presentations.product_id = products.id
                    join organizations on products.organization_id = organizations.id
                    where contents.id = $1 and organizations.id = $2 and products.id = $3 and presentations.id = $4"#)
            .bind(content_id)
            .bind(organization_id)
            .bind(product_id)
            .bind(presentation_id)
            .fetch_optional(&self.pool)
            .await?;

        let Some(content) = content else {
            return Err("no se encuentra el contenido")?;
        };

        sqlx::query("delete from contents where id = $1")
            .bind(content_id)
            .execute(&self.pool)
            .await?;

        self.files
            .delete_by_organization_id_and_id(organization_id, &content.file_id)
            .await?;

        Ok(())
    }

    pub async fn sort(
        &self,
        organization_id: &Id,
        product_id: &Id,
        presentation_id: &Id,
        content_id: &Id,
        new_number: &i64,
    ) -> Result<(), AppError> {
        let content = sqlx::query_as::<_, Content>(r#"
            select contents.id, contents.number, contents.presentation_id, contents.file_id, files.preset as file_preset
                from contents
                join files on contents.file_id = files.id
                join presentations on contents.presentation_id = presentations.id
                join products on presentations.product_id = products.id
                join organizations on products.organization_id = organizations.id
                where organizations.id = $1 and products.id = $2 and presentations.id = $3 and contents.id = $4
        "#)
        .bind(organization_id)
        .bind(product_id)
        .bind(presentation_id)
        .bind(content_id)
        .fetch_one(&self.pool)
        .await?;

        // set current content number to the content that already has the new number
        sqlx::query("update contents set number = $1 where presentation_id = $2 and number = $3")
            .bind(content.number)
            .bind(presentation_id)
            .bind(new_number)
            .execute(&self.pool)
            .await?;

        // set the new number to the current content
        sqlx::query("update contents set number = $1 where id = $2")
            .bind(new_number)
            .bind(content.id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
