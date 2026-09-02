use sqlx::prelude::FromRow;

use crate::infra::Id;

#[derive(FromRow)]
pub struct Content {
    pub id: Id,
    pub number: i64,
    pub presentation_id: Id,
    pub file_id: Id,
    pub file_preset: String,
}
