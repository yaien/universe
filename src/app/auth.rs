mod invitations;
mod oauth;
mod organizations;
mod roles;
mod users;

use std::sync::Arc;

pub use invitations::*;
pub use oauth::*;
pub use organizations::*;
pub use roles::*;
pub use users::*;

use crate::app::sitemaps::Sitemaps;
use crate::infra::DbPool;

pub struct Auth {
    pub oauth: Arc<OAuth>,
    pub users: Arc<Users>,
    pub invitations: Arc<Invitations>,
    pub organizations: Arc<Organizations>,
    pub roles: Arc<Roles>,
}

impl Auth {
    pub fn new(pool: DbPool, sitemaps: Arc<Sitemaps>) -> Self {
        let users = Arc::new(Users::new(pool.clone()));
        let invitations = Arc::new(Invitations::new(pool.clone()));
        let organizations = Arc::new(Organizations::new(pool.clone(), sitemaps));
        let roles = Arc::new(Roles::new(pool.clone()));
        let oauth = Arc::new(OAuth::new(pool, users.clone()));
        Self {
            oauth,
            users,
            invitations,
            organizations,
            roles,
        }
    }
}
