use maud::{DOCTYPE, Markup, PreEscaped, html};

use crate::app::{Color, Layout, Organization, Page, SitemapFont};

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
) -> Markup {
    let content = String::from("content");

    let mut og_description = "";
    let mut og_image = "";
    let mut og_type = "";
    let mut title = "";
    let mut page_css = "";
    let mut page_js = "";

    if let Some(page) = &page {
        og_description = &page.og_description;
        og_image = &page.og_image;
        og_type = &page.og_type;
        title = &page.title;
        page_css = &page.css;
        page_js = &page.js;
    }

    let mut layout_css = "";
    let mut layout_js = "";

    if let Some(layout) = &layout {
        layout_css = &layout.css;
        layout_js = &layout.js;
    }

    html!(
        (DOCTYPE)

        html lang="es" {
            head {
                meta charset="UTF-8" {}
                meta name="viewport" content="width=device-width, initial-scale=1.0" {}
                meta name="theme-color" content="#ffffff" {}
                meta name="description" content=(og_description) {}
                meta name="og:title" content=(title) {}
                meta name="og:description" content=(og_description) {}
                meta name="og:image" content=(format!("{}/assets/external/{}/{}", org.url,org.id, og_image)) {}
                meta name="og:type" content=(og_type) {}

                title {
                    @if title.is_empty() {
                        (org.title)
                    } @else {
                        (title) "-" (org.title)
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
    )
}
