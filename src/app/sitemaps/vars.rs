use sqlx::prelude::FromRow;
use sqlx::types::Json;
use std::collections::HashMap;

use crate::app::sitemaps::contents::{Query, Source};
use crate::app::{AppError, AppResult};
use crate::infra::{DbPool, Id};

#[derive(FromRow)]
pub struct Var {
    pub id: Id,
    pub name: String,
    pub source: Option<Source>,
    pub query: Query,

    #[sqlx(json)]
    pub args: HashMap<String, String>,
}

pub struct Vars {
    pool: DbPool,
}

impl Vars {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_page_id(&self, page_id: &Id) -> AppResult<Vec<Var>> {
        sqlx::query_as::<_, Var>("SELECT * FROM vars where page_id = $1")
            .bind(page_id)
            .fetch_all(&self.pool)
            .await
            .map_err(AppError::Sqlx)
    }

    pub async fn get_by_layout_id(&self, layout_id: &Id) -> AppResult<Vec<Var>> {
        sqlx::query_as::<_, Var>("SELECT * FROM vars where layout_id = $1")
            .bind(layout_id)
            .fetch_all(&self.pool)
            .await
            .map_err(AppError::Sqlx)
    }

    pub async fn get_one_by_id_and_layout_id(&self, var_id: &Id, layout_id: &Id) -> AppResult<Var> {
        sqlx::query_as::<_, Var>("SELECT * FROM vars where id = $1 and layout_id = $2")
            .bind(var_id)
            .bind(layout_id)
            .fetch_one(&self.pool)
            .await
            .map_err(AppError::Sqlx)
    }

    pub async fn get_one_by_id_and_page_id(&self, var_id: &Id, page_id: &Id) -> AppResult<Var> {
        sqlx::query_as::<_, Var>("SELECT * FROM vars where id = $1 and page_id = $2")
            .bind(var_id)
            .bind(page_id)
            .fetch_one(&self.pool)
            .await
            .map_err(AppError::Sqlx)
    }

    pub async fn create_for_page(&self, var: NewPageVar) -> AppResult<Var> {
        if var.args.len() != var.query.required_args().len() * 2 {
            return Err("la cantidad de argumentos no coincide con los requeridos")?;
        }

        sqlx::query_as::<_, Var>("INSERT INTO vars (name, source, query, args, page_id) VALUES ($1, $2, $3, $4, $5) RETURNING *")
            .bind(var.name)
            .bind(var.source)
            .bind(var.query)
            .bind(Json(var.args))
            .bind(var.page_id)
            .fetch_one(&self.pool)
            .await
            .map_err(AppError::Sqlx)
    }

    pub async fn create_for_layout(&self, var: NewLayoutVar) -> AppResult<Var> {
        self.validate_args(&var.query, &var.source, &var.args)?;

        sqlx::query_as::<_, Var>("INSERT INTO vars (name, source, query, args, layout_id) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *")
            .bind(var.name)
            .bind(var.source)
            .bind(var.query)
            .bind(Json(var.args))
            .bind(var.layout_id)
            .fetch_one(&self.pool)
            .await
            .map_err(AppError::Sqlx)
    }

    pub async fn update_for_page(&self, var: UpdatePageVar) -> AppResult<Var> {
        self.validate_args(&var.query, &var.source, &var.args)?;

        sqlx::query_as::<_, Var>("UPDATE vars SET name = $1, source = $2, query = $3, args = $4 WHERE id = $5 and page_id = $6 RETURNING *")
            .bind(var.name)
            .bind(var.source)
            .bind(var.query)
            .bind(Json(var.args))
            .bind(var.var_id)
            .bind(var.page_id)
            .fetch_one(&self.pool)
            .await
            .map_err(AppError::Sqlx)
    }

    pub async fn update_for_layout(&self, var: UpdateLayoutVar) -> AppResult<Var> {
        self.validate_args(&var.query, &var.source, &var.args)?;

        sqlx::query_as::<_, Var>("UPDATE vars SET name = $1, source = $2, query = $3, args = $4 WHERE id = $5 and layout_id = $6 RETURNING *")
            .bind(var.name)
            .bind(var.source)
            .bind(var.query)
            .bind(Json(var.args))
            .bind(var.var_id)
            .bind(var.layout_id)
            .fetch_one(&self.pool)
            .await
            .map_err(AppError::Sqlx)
    }

    fn validate_args(
        &self,
        query: &Query,
        source: &Option<Source>,
        args: &HashMap<String, String>,
    ) -> Result<(), AppError> {
        if query.required_args().is_empty() {
            return Ok(());
        }

        let Some(source) = source else {
            return Err("falta el campo de fuente de argumentos (source)")?;
        };

        for required_arg in query.required_args() {
            match source {
                Source::Static => {
                    if args.get(*required_arg).filter(|k| !k.is_empty()).is_none() {
                        return Err(format!("falta el campo {:?}", *required_arg))?;
                    }
                }

                Source::Query => {
                    if args
                        .get(&format!("query_{}", *required_arg))
                        .filter(|k| !k.is_empty())
                        .is_none()
                    {
                        return Err(format!("falta el campo {:?}", *required_arg))?;
                    }
                }
                _ => {}
            }

            if args
                .get(&format!("test_{}", *required_arg))
                .filter(|k| !k.is_empty())
                .is_none()
            {
                return Err(format!(
                    "falta el campo de pruebas (test_{})",
                    *required_arg
                ))?;
            }
        }

        Ok(())
    }
}

pub struct NewPageVar {
    pub name: String,
    pub source: Option<Source>,
    pub query: Query,
    pub args: HashMap<String, String>,
    pub page_id: Id,
}

pub struct NewLayoutVar {
    pub name: String,
    pub source: Option<Source>,
    pub query: Query,
    pub args: HashMap<String, String>,
    pub layout_id: Id,
}

pub struct UpdatePageVar {
    pub name: String,
    pub source: Option<Source>,
    pub query: Query,
    pub args: HashMap<String, String>,
    pub var_id: Id,
    pub page_id: Id,
}

pub struct UpdateLayoutVar {
    pub name: String,
    pub source: Option<Source>,
    pub query: Query,
    pub args: HashMap<String, String>,
    pub var_id: Id,
    pub layout_id: Id,
}
