use std::sync::Arc;

use crate::app::Storage;
pub use cart::*;
pub use content::*;
pub use order::*;
pub use presentation::*;
pub use product::*;

use crate::infra::DbPool;

mod cart;
mod content;
mod order;
mod presentation;
mod product;

pub struct Store {
    pub products: Products,
    pub presentations: Presentations,
    pub contents: Contents,
    pub carts: Carts,
    pub orders: Orders,
}

impl Store {
    pub fn new(db: DbPool, files: Arc<Storage>) -> Self {
        Self {
            products: Products::new(db.clone()),
            presentations: Presentations::new(db.clone()),
            carts: Carts::new(db.clone()),
            orders: Orders::new(db.clone()),
            contents: Contents::new(db, files),
        }
    }
}
