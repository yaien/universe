use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Context;
use maud::{DOCTYPE, Markup, PreEscaped, html};
use minijinja::{Environment, Error, ErrorKind, Value, context};

use crate::app::auth::{Organization, User};
use crate::app::errors::AppError;
use crate::app::{App, AppResult};
use crate::infra::Id;

use super::{Color, Email, Layout, Page, Sitemap, SitemapFont};

mod args;
mod links;
mod queries;
mod registry;
mod script;
mod style;
mod tools;

pub use args::*;
pub use queries::*;
pub use script::bundle as bundle_js;
pub use style::bundle as bundle_css;
pub use tools::*;

pub use registry::RegisterFunctions;

pub struct RenderPageOptions {
    pub app: Arc<App>,
    pub sitemap: Sitemap,
    pub page: Page,
    pub layout: Option<Layout>,
    pub fonts: Vec<SitemapFont>,
    pub org: Arc<Organization>,
    pub user: Arc<Option<User>>,
    pub query_map: HashMap<String, String>,
    pub request_path: String,
}

/// render_page returns the markup for a page.
pub async fn render_page(options: RenderPageOptions) -> Result<Markup, AppError> {
    let RenderPageOptions {
        app,
        org,
        user,
        page,
        layout,
        sitemap,
        fonts,
        query_map,
        request_path,
    } = options;

    let page_vars = get_page_vars(
        app.clone(),
        org.clone(),
        user.clone(),
        &page,
        &query_map,
        &request_path,
    )
    .await
    .context("failed getting page vars")?;

    let layout_vars = match &layout {
        Some(layout) => get_layout_vars(app.clone(), org.clone(), user.clone(), &layout.id)
            .await
            .context("failed getting layout vars")?,
        None => HashMap::new(),
    };

    let content = get_page_content(&org, &user, &page, &page_vars, &layout, &layout_vars, false)
        .context("failed getting page content")?;

    let page_title = Environment::new()
        .render_str(
            &page.title,
            context! {
                user => user,
                org => org,
                page => page_vars,
                layout => layout_vars,
            },
        )
        .unwrap_or_default();

    Ok(html!(
        (DOCTYPE)

        html lang="es" {
            head {
                meta charset="UTF-8" {}
                meta name="viewport" content="width=device-width, initial-scale=1.0" {}
                meta name="theme-color" content="#ffffff" {}
                meta name="description" content=(page.og_description) {}
                meta name="og:title" content=(page.title) {}
                meta name="og:description" content=(page.og_description) {}
                meta name="og:type" content=(page.og_type) {}

                meta name="htmx-config" content="transitions:true" {}

                @if let Some(og_image_file_id) = page.og_image_file_id {
                    meta name="og:image" content=(format!("{}/assets/dynamic/files/{}", org.url, og_image_file_id)) {}
                    meta property="og:image:type" content="image/webp" {}
                }

                @if let Some(favicon_file_id) = sitemap.favicon_file_id {
                    link rel="icon" href=(format!("/assets/dynamic/favicon.ico?id={}", favicon_file_id)) {}
                }

                title {
                    @if page_title.is_empty() {
                        (org.title)
                    } @else {
                        (page_title) "-" (org.title)
                    }
                }

                (links::fonts(&fonts))

                link rel="stylesheet" href="/assets/landing/style.css" {}

                script defer src="https://cdn.jsdelivr.net/npm/alpinejs@3.x.x/dist/cdn.min.js" {}
                script src="https://cdn.jsdelivr.net/npm/htmx.org@4.0.0/dist/htmx.min.js" {}
                script src="/assets/landing/script.js" {}

            }

            body {
                div data-layout=(layout.as_ref().map_or("", |l| l.name.as_str())) {
                    (PreEscaped(content))
                }
            }
        }
    ))
}

pub struct RenderPageInlineOptions {
    pub app: Arc<App>,
    pub page: Page,
    pub layout: Option<Layout>,
    pub fonts: Vec<SitemapFont>,
    pub colors: Vec<Color>,
    pub user: Arc<Option<User>>,
    pub org: Arc<Organization>,
}

pub async fn render_page_inline(options: RenderPageInlineOptions) -> Result<Markup, AppError> {
    let RenderPageInlineOptions {
        app,
        page,
        layout,
        fonts,
        colors,
        user,
        org,
    } = options;

    let page_vars = get_page_inline_vars(app.clone(), org.clone(), user.clone(), &page.id)
        .await
        .context("failed getting page vars")?;

    let layout_vars = match &layout {
        Some(layout) => get_layout_vars(app.clone(), org.clone(), user.clone(), &layout.id).await?,
        None => HashMap::new(),
    };

    let mut content = get_page_content(&org, &user, &page, &page_vars, &layout, &layout_vars, true)
        .context("failed getting page content")?;

    let page_title_context = context! {
        user => user,
        org => org,
        page => page_vars,
        layout => layout_vars,
    };

    let page_title = match Environment::new().render_str(&page.title, page_title_context) {
        Ok(title) => title,
        Err(e) => {
            content = render_template_error("failed at page title render", e);
            page.title.clone()
        }
    };

    Ok(html!(
        (DOCTYPE)

        html lang="es" {
            head hx-head="merge" {
                meta charset="UTF-8" {}
                meta name="viewport" content="width=device-width, initial-scale=1.0" {}
                meta name="htmx-config" content="transitions:true" {}
                title { (page_title) }

                (links::fonts(&fonts))

                @let layout_css = layout.as_ref().map_or("", |l| l.css.as_str());
                (style::inline(&fonts, &colors, layout_css, &page.css))

                @let layout_js = layout.as_ref().map_or("", |l| l.js.as_str());
                (script::inline(layout_js, &page.js))

                script defer src="https://cdn.jsdelivr.net/npm/alpinejs@3.x.x/dist/cdn.min.js" {}
                script src="https://cdn.jsdelivr.net/npm/htmx.org@4.0.0/dist/htmx.min.js" {}
                script src="https://cdn.jsdelivr.net/npm/htmx.org@4.0.0/dist/ext/hx-head.min.js" {}
                script src="https://cdn.jsdelivr.net/npm/htmx.org@4.0.0/dist/ext/hx-alpine-compat.js" {}
            }

            body hx-trigger="reload" hx-get="/dashboard/sitemaps/preview" {
                div data-layout=(layout.as_ref().map_or("", |l| l.name.as_str())) {
                    (PreEscaped(content))
                }
            }
        }
    ))
}

fn render_template_error(prelude: &str, e: minijinja::Error) -> String {
    html!(
      #error style="padding: 1rem; box-sizing: border-box;" {
          pre {
              (format!("{prelude}: {e:#}"))
          }
      }
    )
    .into_string()
}

async fn get_page_vars(
    app: Arc<App>,
    org: Arc<Organization>,
    user: Arc<Option<User>>,
    page: &Page,
    query: &HashMap<String, String>,
    request_path: &str,
) -> AppResult<HashMap<String, Value>> {
    let mut map = HashMap::new();
    let vars = app.sitemaps.vars.get_by_page_id(&page.id).await?;
    for var in vars {
        let args = match &var.source {
            Some(source) => source.map_args(
                query,
                &page.path,
                request_path,
                &var.args,
                var.query.required_args(),
            )?,
            None => HashMap::new(),
        };

        let result = var.query.call(&app, &org, &user, &args).await?;

        map.insert(var.name.clone(), result);
    }
    Ok(map)
}

async fn get_page_inline_vars(
    app: Arc<App>,
    org: Arc<Organization>,
    user: Arc<Option<User>>,
    page_id: &Id,
) -> AppResult<HashMap<String, Value>> {
    let vars = app.sitemaps.vars.get_by_page_id(&page_id).await?;
    let mut map = HashMap::new();
    for var in vars {
        let result = var.query.call_inline(&app, &org, &user, &var.args).await?;

        map.insert(var.name.clone(), result);
    }
    Ok(map)
}

fn get_page_content(
    org: &Organization,
    user: &Option<User>,
    page: &Page,
    page_vars: &HashMap<String, Value>,
    layout: &Option<Layout>,
    layout_vars: &HashMap<String, Value>,
    inline: bool,
) -> Result<String, AppError> {
    let layout_template = layout
        .as_ref()
        .map_or("{% block body %}{% endblock %}", |l| l.html.as_str());

    log::info!("page_id {}", page.id);

    let mut env = Environment::new();

    env.register_functions(inline);

    match env.add_template("layout", layout_template) {
        Ok(_) => {}
        Err(e) => return Ok(render_template_error("failed on layout template", e)),
    };

    match env.add_template("page", &page.html) {
        Ok(templ) => templ,
        Err(e) => return Ok(render_template_error("failed on page template", e)),
    };

    let content_template = format!(
        r#"
          {{% extends "layout"%}}
            {{% block body %}}
                <div data-page={:?}>
                    {{% include "page" %}}
                </div>
            {{% endblock %}}
          "#,
        page.name
    );

    let s = context! {
        user => user,
        org => org,
        layout => layout_vars,
        page => page_vars,
    };

    let content = match env.render_str(&content_template, s) {
        Ok(content) => content,
        Err(e) => return Ok(render_template_error("failed on render", e)),
    };

    Ok(content)
}

pub struct RenderLayoutOptions {
    pub app: Arc<App>,
    pub layout: Layout,
    pub fonts: Vec<SitemapFont>,
    pub colors: Vec<Color>,
    pub user: Arc<Option<User>>,
    pub org: Arc<Organization>,
}

/// render_layout returns the markup for a layout.
pub async fn render_layout(options: RenderLayoutOptions) -> Result<Markup, AppError> {
    let RenderLayoutOptions {
        app,
        layout,
        fonts,
        colors,
        org,
        user,
    } = options;

    let layout_vars = get_layout_vars(app.clone(), org.clone(), user.clone(), &layout.id).await?;

    let content =
        get_layout_content(&org, &user, &layout, &layout_vars).context("failed getting content")?;

    Ok(html!(
        (DOCTYPE)

        html lang="es" {
             head hx-head="merge" {
                 meta charset="UTF-8" {}
                 meta name="viewport" content="width=device-width, initial-scale=1.0" {}
                 meta name="theme-color" content="#ffffff" {}
                 meta name="htmx-config" content="transitions:true" {}

                 (links::fonts(&fonts))
                 (style::inline(&fonts, &colors, &layout.css, ""))
                 (script::inline(&layout.js, ""))

                 script defer src="https://cdn.jsdelivr.net/npm/alpinejs@3.x.x/dist/cdn.min.js" {}
                 script src="https://cdn.jsdelivr.net/npm/htmx.org@4.0.0/dist/htmx.min.js" {}
                 script src="https://cdn.jsdelivr.net/npm/htmx.org@4.0.0/dist/ext/hx-head.min.js" {}
                 script src="https://cdn.jsdelivr.net/npm/htmx.org@4.0.0/dist/ext/hx-alpine-compat.js" {}
            }
            body hx-trigger="reload" hx-get="/dashboard/sitemaps/preview" {
                div data-layout=(layout.name) {
                    (PreEscaped(content))
                }
            }
        }
    ))
}

async fn get_layout_vars(
    app: Arc<App>,
    org: Arc<Organization>,
    user: Arc<Option<User>>,
    layout_id: &Id,
) -> AppResult<HashMap<String, Value>> {
    let mut map = HashMap::new();
    let vars = app.sitemaps.vars.get_by_layout_id(layout_id).await?;
    for var in vars {
        let value = var.query.call_inline(&app, &org, &user, &var.args).await?;
        map.insert(var.name, value);
    }

    Ok(map)
}

fn get_layout_content(
    org: &Organization,
    user: &Option<User>,
    layout: &Layout,
    layout_vars: &HashMap<String, Value>,
) -> Result<String, AppError> {
    let mut env = Environment::new();

    env.register_functions(true);

    let s = context! { org => org, user => user, layout => layout_vars };

    let content = match env.render_str(&layout.html, s) {
        Ok(content) => content,
        Err(e) => return Ok(render_template_error("failed on render", e)),
    };

    Ok(content)
}

pub struct EmailRegistryContext<'a> {
    pub org: &'a Organization,
}

/// get_email_content returns the subject and body of an email.
pub fn get_email_content<'a>(
    ctx: &EmailRegistryContext<'a>,
    email: &Email,
) -> Result<(String, String), AppError> {
    let env = Environment::new();

    let ctx = context! { org => &ctx.org };

    let subject = match env.render_str(&email.subject, &ctx) {
        Ok(subject) => subject,
        Err(e) => return Ok((render_template_error("failed on render", e), String::new())),
    };

    let body = match env.render_str(&email.body, &ctx) {
        Ok(body) => body,
        Err(e) => return Ok((subject, render_template_error("failed on render", e))),
    };

    Ok((subject, body))
}

pub fn render_email(ctx: EmailRegistryContext<'_>, email: &Email) -> Result<Markup, AppError> {
    let (subject, body) = get_email_content(&ctx, email)?;

    Ok(html!(
        (DOCTYPE)
        html lang="es" {
            head {
                meta charset="UTF-8" {}
                meta name="viewport" content="width=device-width, initial-scale=1.0" {}
                meta name="theme-color" content="#ffffff" {}
                style type="text/css" {
                    (PreEscaped(r#"
                        .subject {
                            width: 100%;
                            position: absolute;
                            bottom: 0;
                            left: 0;
                            z-index: 1;
                        }

                        body {
                            width: 100%;
                            height: 100%;
                            margin: 0;
                        }

                    "#))
                }
            }
            body {
                .subject { (subject) }
                (PreEscaped(body))
            }
        }
    ))
}
