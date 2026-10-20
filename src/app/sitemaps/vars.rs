use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sqlx::prelude::{FromRow, Type};
use sqlx::types::Json;
use strum_macros::{Display, EnumIter};

use crate::app::sitemaps::contents::Query;
use crate::app::{AppError, AppResult};
use crate::infra::{DbPool, Id};

#[derive(Type, Deserialize, Serialize, PartialEq, EnumIter, Display)]
pub enum Source {
    #[strum(to_string = "Cadena de consulta")]
    Query,

    #[strum(to_string = "Ruta")]
    Path,

    #[strum(to_string = "Estatico")]
    Static,
}

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
        if var.args.len() != var.query.required_args().len() {
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
        if var.args.len() != var.query.required_args().len() {
            return Err("la cantidad de argumentos no coincide con los requeridos")?;
        }
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
