use std::sync::Arc;

use minijinja::Environment;

use crate::app::{App, Organization, User};

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
    fn register_functions(&mut self, ctx: &RegistryContext);
}

impl RegisterFunctions for Environment<'_> {
    fn register_functions(&mut self, RegistryContext { app, org, user }: &RegistryContext) {
        register!(file_url, self);
        register!(external_file_url, self, org);
    }
}

fn file_url(name: String, variant: Option<String>) -> String {
    match variant {
        Some(variant) => format!("/assets/dynamic/files/{name}?v={variant}"),
        None => format!("/assets/dynamic/files/{name}"),
    }
}

fn external_file_url(org: Arc<Organization>) -> impl Fn(String, Option<String>) -> String {
    move |name: String, variant: Option<String>| -> String {
        match variant {
            Some(variant) => {
                format!("{}/assets/dynamic/files/{name}?v={variant}", org.url)
            }
            None => format!("{}/assets/dynamic/files/{name}", org.url),
        }
    }
}
