pub mod auth;
mod errors;
pub mod sitemaps;
pub mod storage;
pub mod store;

use std::sync::Arc;

use auth::Auth;
use sitemaps::Sitemaps;
use storage::Storage;
use store::Store;

use crate::infra::Monolith;

pub use errors::*;

pub struct App {
    pub sitemaps: Arc<Sitemaps>,
    pub auth: Arc<Auth>,
    pub storage: Arc<Storage>,
    pub store: Arc<Store>,
}

impl App {
    pub fn new(mono: &Monolith) -> Self {
        let sitemaps = Arc::new(Sitemaps::new(mono.pool.clone()));

        let auth = Arc::new(Auth::new(mono.pool.clone(), sitemaps.clone()));

        let storage = Arc::new(Storage::new(
            mono.pool.clone(),
            mono.queue.clone(),
            mono.config.storage_path.clone(),
        ));

        let store = Arc::new(Store::new(mono.pool.clone(), storage.clone()));

        Self {
            sitemaps,
            auth,
            storage,
            store,
        }
    }
}
