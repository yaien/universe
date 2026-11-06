use minijinja::Environment;

macro_rules! register {
    ($func: ident, $env: expr) => {
        $env.add_function(stringify!($func), $func);
    };
    ($func: ident, $env: expr, $($ctx: ident),*) => {
        $env.add_function(stringify!($func), $func($($ctx.clone()),*));
    };
}

pub trait RegisterFunctions {
    fn register_functions(&mut self, inline: bool);
}

impl RegisterFunctions for Environment<'_> {
    fn register_functions(&mut self, inline: bool) {
        register!(file_url, self);
        register!(action_url, self, inline);
    }
}

fn file_url(name: String, variant: Option<String>) -> String {
    match variant {
        Some(variant) => format!("/assets/dynamic/files/{name}?variant={variant}"),
        None => format!("/assets/dynamic/files/{name}"),
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
