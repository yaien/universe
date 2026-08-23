mod auth;
mod color;
mod email;
mod error;
mod file;
mod font;
mod integration;
mod invitation;
mod layout;
mod organization;
mod page;
mod role;
mod sitemap;
mod user;

pub mod file_conversions;
pub mod file_dimensions;
pub mod file_processor;
pub mod file_state;

pub use auth::*;
pub use color::*;
pub use email::*;
pub use error::*;
pub use file::*;
pub use font::*;
pub use integration::*;
pub use invitation::*;
pub use layout::*;
pub use organization::*;
pub use page::*;
pub use role::*;
pub use sitemap::*;
pub use user::*;

use crate::infra::Monolith;
use std::sync::Arc;

pub struct App {
    pub organizations: Arc<Organizations>,
    pub pages: Arc<Pages>,
    pub sitemaps: Arc<Sitemaps>,
    pub emails: Arc<Emails>,
    pub layouts: Arc<Layouts>,
    pub auth: Arc<Auth>,
    pub users: Arc<Users>,
    pub invitations: Arc<Invitations>,
    pub roles: Arc<Roles>,
    pub files: Arc<Files>,
    pub fonts: Arc<Fonts>,
    pub colors: Arc<Colors>,
}

impl App {
    pub fn new(mono: &Monolith) -> Self {
        let users = Arc::new(Users::new(mono.pool.clone()));

        let pages = Arc::new(Pages::new(mono.pool.clone()));

        let emails = Arc::new(Emails::new(mono.pool.clone()));

        let layouts = Arc::new(Layouts::new(mono.pool.clone()));

        let fonts = Arc::new(Fonts::new(mono.pool.clone()));

        let colors = Arc::new(Colors::new(mono.pool.clone()));

        let sitemaps = Arc::new(Sitemaps::new(
            mono.pool.clone(),
            pages.clone(),
            emails.clone(),
            fonts.clone(),
            colors.clone(),
            layouts.clone(),
        ));

        let invitations = Arc::new(Invitations::new(mono.pool.clone()));

        let roles = Arc::new(Roles::new(mono.pool.clone()));

        let organizations = Arc::new(Organizations::new(mono.pool.clone(), sitemaps.clone()));

        let auth = Arc::new(Auth::new(mono.pool.clone(), users.clone()));

        let files = Arc::new(Files::new(
            mono.pool.clone(),
            mono.queue.clone(),
            mono.config.storage_path.clone(),
        ));

        Self {
            users,
            pages,
            sitemaps,
            emails,
            layouts,
            organizations,
            auth,
            invitations,
            roles,
            files,
            fonts,
            colors,
        }
    }
}
