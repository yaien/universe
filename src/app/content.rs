use anyhow::Context;
use maud::{DOCTYPE, Markup, PreEscaped, html};
use minijinja::{Environment, Value, context};

use crate::app::{AppError, Color, Email, Layout, Page, SitemapFont};

mod links;
mod registry;
mod script;
mod style;

pub use script::bundle as bundle_js;
pub use style::bundle as bundle_css;

pub use registry::{RegisterFunctions, RegistryContext};

pub enum RenderMode {
    External,
    Inline { colors: Vec<Color> },
}

pub struct RenderPageOptions {
    pub page: Page,
    pub layout: Option<Layout>,
    pub fonts: Vec<SitemapFont>,
    pub mode: RenderMode,
    pub ctx: RegistryContext,
}

/// render_page returns the markup for a page.
pub fn render_page(options: RenderPageOptions) -> Result<Markup, AppError> {
    let RenderPageOptions {
        ctx,
        page,
        layout,
        fonts,
        mode,
    } = options;

    let content = get_page_content(&ctx, &page, &layout).context("failed getting page content")?;

    let org = &ctx.org;

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
                    meta name="og:image" content=(format!("{}/assets/external/{}/{}", org.url, org.id, og_image_file_id)) {}
                }

                title {
                    @if page.title.is_empty() {
                        (org.title)
                    } @else {
                        (page.title) "-" (org.title)
                    }
                }



                (links::fonts(&fonts))

                link rel="icon" type="image/png" href="/assets/landing/favicon.png" {}


                @match mode {
                    RenderMode::External => {
                        link rel="stylesheet" href="/assets/landing/style.css" {}
                        script defer src="/assets/landing/script.js" {}
                    }
                    RenderMode::Inline { colors } => {
                        @let layout_css = layout.as_ref().map_or("", |l| l.css.as_str());
                        @let layout_js = layout.as_ref().map_or("", |l| l.js.as_str());

                        (style::inline(&fonts, &colors, layout_css, &page.css))
                        (script::inline(layout_js, &page.js))
                    }
                }


                script defer src="https://cdn.jsdelivr.net/npm/alpinejs@3.x.x/dist/cdn.min.js" {}
                script src="https://cdn.jsdelivr.net/npm/htmx.org@4.0.0-beta6" integrity="sha384-6lyVbhrs13b9z7mLOpt/N6R76rtkEBWgCjAXRs/DSWyi2AMnQSs10ijWk+PI8n7W" crossorigin="anonymous" {}

            }

            body {
                div data-layout=(layout.as_ref().map_or("", |l| l.name.as_str())) {
                    (PreEscaped(content))
                }
            }
        }
    ))
}

fn get_page_content(
    ctx: &RegistryContext,
    page: &Page,
    layout: &Option<Layout>,
) -> Result<String, AppError> {
    let layout_template = layout
        .as_ref()
        .map_or("{% block body %}{% endblock %}", |l| l.html.as_str());

    log::info!("page_id {}", page.id);

    let page_template = format!(
        r#"
          {{% extends "layout"%}}
            {{% block body %}}
                <div data-page={:?}>
                    {}
                </div>
            {{% endblock %}}
          "#,
        page.name, page.html
    );

    let mut env = Environment::new();

    env.register_functions(&ctx);

    env.add_template("layout", layout_template)
        .context("failed adding layout template")?;

    let templ = env
        .template_from_named_str("page", &page_template)
        .context("failed adding page template")?;

    let s = context! {user => &ctx.user, org => &ctx.org};

    let content = templ.render(s).context("failed rendering template")?;

    Ok(content)
}

pub struct RenderLayoutOptions {
    pub layout: Layout,
    pub fonts: Vec<SitemapFont>,
    pub colors: Vec<Color>,
    pub ctx: RegistryContext,
}

/// render_layout returns the markup for a layout.
pub fn render_layout(options: RenderLayoutOptions) -> Result<Markup, AppError> {
    let RenderLayoutOptions {
        layout,
        fonts,
        colors,
        ctx,
    } = options;

    let content = get_layout_content(&ctx, &layout).context("failed getting content")?;

    Ok(html!(
        (DOCTYPE)

        html lang="es" {
             head {
                 meta charset="UTF-8" {}
                 meta name="viewport" content="width=device-width, initial-scale=1.0" {}
                 meta name="theme-color" content="#ffffff" {}

                 (links::fonts(&fonts))
                 (style::inline(&fonts, &colors, &layout.css, ""))
                 (script::inline(&layout.js, ""))
            }
            body {
                div data-layout=(layout.name) {
                    (PreEscaped(content))
                }
            }
        }
    ))
}

fn get_layout_content(ctx: &RegistryContext, layout: &Layout) -> Result<String, AppError> {
    let mut env = Environment::new();

    env.register_functions(ctx);

    let s = context! { org => &ctx.org, user => &ctx.user };

    let content = env
        .render_str(&layout.html, s)
        .context("failed rendering template")?;

    Ok(content)
}

/// get_email_content returns the subject and body of an email.
pub fn get_email_content(email: &Email, ctx: Value) -> Result<(String, String), AppError> {
    let env = Environment::new();

    let subject = env
        .render_str(&email.subject, &ctx)
        .context("failed rendering subject")?;

    let body = env
        .render_str(&email.body, &ctx)
        .context("failed rendering body")?;

    Ok((subject, body))
}

pub fn render_email(email: &Email, ctx: Value) -> Result<Markup, AppError> {
    let (subject, body) = get_email_content(email, ctx)?;

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
