use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;

use crate::infra::Id;

#[derive(FromRow)]
pub struct File {
    pub id: Id,
    pub organization_id: Id,
    pub name: String,
    pub preset: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(FromRow)]
pub struct FileFormat {
    pub id: Id,
    pub file_id: Id,
    pub variant: i32,
    pub size: usize,
    pub width: i32,
    pub height: i32,
    pub content_type: String,
}
