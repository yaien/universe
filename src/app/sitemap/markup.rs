use anyhow::Context;
use maud::{DOCTYPE, Markup, PreEscaped, html};
use minijinja::{Environment, context};

use crate::app::{AppError, Color, Layout, Organization, Page, SitemapFont};

use super::{links, script, style};

pub enum Mode {
    External,
    Inline { colors: Vec<Color> },
}

pub struct MarkupOptions<'a> {
    pub org: &'a Organization,
    pub page: Option<Page>,
    pub layout: Option<Layout>,
    pub fonts: Vec<SitemapFont>,
    pub mode: Mode,
}

pub fn markup(
    MarkupOptions {
        org,
        page,
        layout,
        fonts,
        mode,
    }: MarkupOptions<'_>,
) -> Result<Markup, AppError> {
    let layout_template = layout
        .as_ref()
        .map_or("{% block body %}{% endblock %}", |l| l.html.as_str());

    let page_content = page.as_ref().map(|p| p.html.as_str()).unwrap_or("");

    let page_template = format!(
        r#"
        {{% extends "layout"%}}
        {{% block body %}}
            {page_content}
        {{% endblock %}}
        "#
    );

    let mut env = Environment::new();

    env.add_template("layout", layout_template)
        .context("failed adding layout template")?;

    let templ = env
        .template_from_named_str("page", &page_template)
        .context("failed adding page template")?;

    let ctx = context! {};

    let content = templ.render(ctx).context("failed rendering template")?;

    Ok(html!(
        (DOCTYPE)

        html lang="es" {
            head {
                meta charset="UTF-8" {}
                meta name="viewport" content="width=device-width, initial-scale=1.0" {}
                meta name="theme-color" content="#ffffff" {}

                @if let Some(page) = &page {
                    meta name="description" content=(page.og_description) {}
                    meta name="og:title" content=(page.title) {}
                    meta name="og:description" content=(page.og_description) {}
                    meta name="og:type" content=(page.og_type) {}

                    @if !page.og_image.is_empty() {
                        meta name="og:image" content=(format!("{}/assets/external/{}/{}", org.url, org.id, page.og_image)) {}
                    }

                    title {
                        @if page.title.is_empty() {
                            (org.title)
                        } @else {
                            (page.title) "-" (org.title)
                        }
                    }
                }



                (links::fonts(&fonts))

                link rel="icon" type="image/png" href="/assets/landing/favicon.png" {}

                @match mode {
                    Mode::External => {
                        link rel="stylesheet" href="/assets/landing/styles.css" {}
                        script type="text/javascript" src="/assets/landing/script.js" {}
                    }
                    Mode::Inline { colors } => {
                        @let layout_css = layout.as_ref().map_or("", |l| l.css.as_str());
                        @let layout_js = layout.as_ref().map_or("", |l| l.js.as_str());
                        @let page_css = page.as_ref().map_or("", |p| p.css.as_str());
                        @let page_js = page.as_ref().map_or("", |p| p.js.as_str());

                        (style::inline(&fonts, &colors, layout_css, page_css))
                        (script::inline(layout_js, page_js))
                    }
                }

                script defer src="https://cdn.jsdelivr.net/npm/alpinejs@3.x.x/dist/cdn.min.js" {}
                script src="https://cdn.jsdelivr.net/npm/htmx.org@4.0.0-beta6" integrity="sha384-6lyVbhrs13b9z7mLOpt/N6R76rtkEBWgCjAXRs/DSWyi2AMnQSs10ijWk+PI8n7W" crossorigin="anonymous" {}
            }

            body {
                (PreEscaped(content))
            }
        }
    ))
}
