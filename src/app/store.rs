use sqlx::prelude::FromRow;

pub use crate::app::store::content::*;
pub use crate::app::store::presentation::*;
pub use crate::app::store::product::*;
use crate::infra::{DbPool, Id};

mod content;
mod presentation;
mod product;

pub struct Store {
    pub products: Products,
    pub presentations: Presentations,
}

impl Store {
    pub fn new(db: DbPool) -> Self {
        Self {
            products: Products::new(db.clone()),
            presentations: Presentations::new(db),
        }
    }
}
