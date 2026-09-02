use std::sync::Arc;

use crate::app::Files;
pub use crate::app::store::content::*;
pub use crate::app::store::presentation::*;
pub use crate::app::store::product::*;
use crate::infra::DbPool;

mod content;
mod presentation;
mod product;

pub struct Store {
    pub products: Products,
    pub presentations: Presentations,
    pub contents: Contents,
}

impl Store {
    pub fn new(db: DbPool, files: Arc<Files>) -> Self {
        Self {
            products: Products::new(db.clone()),
            presentations: Presentations::new(db.clone()),
            contents: Contents::new(db, files),
        }
    }
}
