mod auth;
mod email;
mod layout;
mod organization;
mod page;
mod sitemap;
mod user;

use std::sync::Arc;

pub use auth::*;
pub use email::*;
pub use layout::*;
pub use organization::*;
pub use page::*;
pub use sitemap::*;
pub use user::*;

use crate::infra::Monolith;

pub struct App {
    pub organizations: Arc<Organizations>,
    pub pages: Arc<Pages>,
    pub sitemaps: Arc<Sitemaps>,
    pub emails: Arc<Emails>,
    pub layouts: Arc<Layouts>,
    pub auth: Arc<Auth>,
    pub users: Arc<Users>,
}

impl App {
    pub fn new(mono: &Monolith) -> Self {
        let users = Arc::new(Users::new(mono.pool.clone()));

        let pages = Arc::new(Pages::new(mono.pool.clone()));

        let sitemaps = Arc::new(Sitemaps::new(mono.pool.clone()));

        let emails = Arc::new(Emails::new(mono.pool.clone()));

        let layouts = Arc::new(Layouts::new(mono.pool.clone()));

        let organizations = Arc::new(Organizations::new(
            mono.pool.clone(),
            sitemaps.clone(),
            pages.clone(),
            emails.clone(),
            layouts.clone(),
        ));

        let auth = Arc::new(Auth::new(mono.pool.clone(), users.clone()));

        Self {
            users,
            pages,
            sitemaps,
            emails,
            layouts,
            organizations,
            auth,
        }
    }
}
