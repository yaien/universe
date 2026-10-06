mod fields;
mod submissions;

use std::collections::HashMap;

pub use fields::*;
pub use submissions::*;

use sqlx::prelude::FromRow;

use crate::app::{AppError, AppResult};
use crate::infra::{DbPool, Id};

#[derive(FromRow)]
pub struct Form {
    pub id: Id,
    pub name: String,
    pub codename: String,

    #[sqlx(skip)]
    pub fields: Vec<FormField>,
}

pub struct Forms {
    pool: DbPool,

    pub fields: FormFields,
    pub submissions: FormSubmissions,
}

impl Forms {
    pub fn new(pool: DbPool) -> Self {
        let fields = FormFields::new(pool.clone());
        let submissions = FormSubmissions::new(pool.clone());
        Self {
            pool,
            fields,
            submissions,
        }
    }

    pub async fn create(&self, org_id: &Id, name: &str, codename: &str) -> AppResult<Form> {
        let form = sqlx::query_as::<_, Form>(
            "insert into forms (organization_id, name, codename) values ($1, $2, $3) returning *",
        )
        .bind(org_id)
        .bind(name)
        .bind(codename)
        .fetch_one(&self.pool)
        .await?;

        Ok(form)
    }

    pub async fn update(
        &self,
        org_id: &Id,
        form_id: &Id,
        name: &str,
        codename: &str,
    ) -> AppResult<Form> {
        let form = sqlx::query_as::<_, Form>(
            "update forms set name = $1, codename = $2 where organization_id = $3 and id = $4 returning *",
        )
        .bind(name)
        .bind(codename)
        .bind(org_id)
        .bind(form_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(form)
    }

    pub async fn get_by_organization_id(&self, org_id: &Id) -> AppResult<Vec<Form>> {
        sqlx::query_as::<_, Form>(
            "select * from forms where organization_id = $1 and deleted_at is null",
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::Sqlx)
    }

    pub async fn get_one_by_organization_id(&self, org_id: &Id, form_id: &Id) -> AppResult<Form> {
        let mut form = sqlx::query_as::<_, Form>(
            "select * from forms where organization_id = $1 and id = $2 and deleted_at is null",
        )
        .bind(org_id)
        .bind(form_id)
        .fetch_one(&self.pool)
        .await?;

        let fields = sqlx::query_as::<_, FormField>(
            "select * from form_fields where form_id = $1 and deleted_at is null order by number",
        )
        .bind(form_id)
        .fetch_all(&self.pool)
        .await?;

        form.fields = fields;

        Ok(form)
    }

    pub async fn submit(
        &self,
        org_id: &Id,
        form_id: &Id,
        data: HashMap<String, String>,
    ) -> AppResult<()> {
        let form = self.get_one_by_organization_id(org_id, form_id).await?;

        // validate data against form fields
        for field in &form.fields {
            if data.get(&field.name).filter(|f| f.is_empty()).is_some() {
                return Err(format!("el campo {:?} es requerido", field.name))?;
            }
        }

        // insert submission
        let submission_id = sqlx::query("insert into form_submissions (form_id) values ($1)")
            .bind(form_id)
            .execute(&self.pool)
            .await
            .map(|r| r.last_insert_rowid())?;

        for field in &form.fields {
            let value = data.get(&field.name);

            sqlx::query("insert into form_submission_answers (submission_id, field_id, value) values ($1, $2, $3)")
                .bind(submission_id)
                .bind(field.id)
                .bind(value)
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }
}
