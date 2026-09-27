use std::sync::Arc;

use minijinja::Environment;

use crate::app::App;
use crate::app::auth::{Organization, User};

macro_rules! register {
    ($func: ident, $env: expr) => {
        $env.add_function(stringify!($func), $func);
    };
    ($func: ident, $env: expr, $($ctx: ident),*) => {
        $env.add_function(stringify!($func), $func($($ctx.clone()),*));
    };
}

pub struct RegistryContext {
    pub app: Arc<App>,
    pub org: Arc<Organization>,
    pub user: Arc<Option<User>>,
}

pub trait RegisterFunctions {
    fn register_functions(&mut self, ctx: &RegistryContext, inline: bool);
}

impl RegisterFunctions for Environment<'_> {
    fn register_functions(&mut self, ctx: &RegistryContext, inline: bool) {
        let RegistryContext {
            app: _,
            org,
            user: _,
        } = ctx;

        register!(file_url, self);
        register!(external_file_url, self, org);
        register!(action_url, self, inline);
    }
}

fn file_url(name: String, variant: Option<String>) -> String {
    match variant {
        Some(variant) => format!("/assets/dynamic/files/{name}?variant={variant}"),
        None => format!("/assets/dynamic/files/{name}"),
    }
}

fn external_file_url(org: Arc<Organization>) -> impl Fn(String, Option<String>) -> String {
    move |name: String, variant: Option<String>| -> String {
        match variant {
            Some(variant) => {
                format!("{}/assets/dynamic/files/{name}?variant={variant}", org.url)
            }
            None => format!("{}/assets/dynamic/files/{name}", org.url),
        }
    }
}

fn action_url(inline: bool) -> impl Fn(String) -> String {
    move |codename: String| -> String {
        match inline {
            true => format!("/dashboard/sitemaps/__actions/{}", codename),
            false => format!("/__actions/{}", codename),
        }
    }
}
