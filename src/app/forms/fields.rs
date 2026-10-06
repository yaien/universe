use chrono::Utc;
use sqlx::prelude::FromRow;

use crate::app::AppResult;
use crate::infra::{DbPool, Id};

#[derive(FromRow)]
pub struct FormField {
    pub id: Id,
    pub name: String,
    pub label: String,
}

pub struct FormFields {
    pool: DbPool,
}

impl FormFields {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        org_id: &Id,
        form_id: &Id,
        name: &str,
        label: &str,
    ) -> AppResult<FormField> {
        // check if the form exists
        let form_exists: bool = sqlx::query_scalar("select exists(select 1 from forms where organization_id = $1 and id = $2 and deleted_at is null)")
            .bind(org_id)
            .bind(form_id)
            .fetch_one(&self.pool)
            .await?;

        if !form_exists {
            return Err("no se encuentra el formulario")?;
        }

        // check it the name and form_id are unique
        let field_exists: bool = sqlx::query_scalar("select exists(select 1 from form_fields where form_id = $1 and name = $2 and deleted_at is null)")
            .bind(form_id)
            .bind(name)
            .fetch_one(&self.pool)
            .await?;

        if field_exists {
            return Err("el nombre del campo ya existe")?;
        }

        // get max number
        let max_number: i32 = sqlx::query_scalar("select coalesce(max(number), 0) from form_fields where form_id = $1 and deleted_at is null")
            .bind(form_id)
            .fetch_one(&self.pool)
            .await?;

        let number = max_number + 1;

        // create the field
        let field: FormField = sqlx::query_as("insert into form_fields (form_id, name, label, number) values ($1, $2, $3, $4) returning *")
            .bind(form_id)
            .bind(name)
            .bind(label)
            .bind(number)
            .fetch_one(&self.pool)
            .await?;

        Ok(field)
    }

    pub async fn update(
        &self,
        org_id: &Id,
        form_id: &Id,
        field_id: &Id,
        name: &str,
        label: &str,
    ) -> AppResult<FormField> {
        // check if the form exists
        let form_exists: bool = sqlx::query_scalar("select exists(select 1 from forms where organization_id = $1 and id = $2 and deleted_at is null)")
            .bind(org_id)
            .bind(form_id)
            .fetch_one(&self.pool)
            .await?;

        if !form_exists {
            return Err("no se encuentra el formulario")?;
        }

        // check if there is already a field with the same name
        let field_exists: bool = sqlx::query_scalar("select exists(select 1 from form_fields where form_id = $1 and id != $2 and name = $3 and deleted_at is null)")
            .bind(form_id)
            .bind(field_id)
            .bind(name)
            .fetch_one(&self.pool)
            .await?;

        if field_exists {
            return Err("ya existe un campo con el mismo nombre")?;
        }

        // update the field
        let field: FormField = sqlx::query_as("update form_fields set name = $1, label = $2 where form_id = $3 and id = $4 returning *")
            .bind(name)
            .bind(label)
            .bind(form_id)
            .bind(field_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(field)
    }

    pub async fn delete(&self, org_id: &Id, form_id: &Id, field_id: &Id) -> AppResult<()> {
        // check if the form exists
        let form_exists: bool = sqlx::query_scalar("select exists(select 1 from forms where organization_id = $1 and id = $2 and deleted_at is null)")
            .bind(org_id)
            .bind(form_id)
            .fetch_one(&self.pool)
            .await?;

        if !form_exists {
            return Err("no se encuentra el formulario")?;
        }

        // delete the field
        sqlx::query("update form_fields set deleted_at = $1 where form_id = $2 and id = $3")
            .bind(Utc::now())
            .bind(form_id)
            .bind(field_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
