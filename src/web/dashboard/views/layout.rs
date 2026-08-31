use derive_more::Display;
use maud::{DOCTYPE, Markup, html};

use crate::app::{Organization, Role};

#[derive(Display)]
#[display(rename_all = "lowercase")]
pub enum Variant {
    Primary,
}

pub struct Content<'a> {
    pub title: &'a str,
    pub path: &'a str,
    pub org: &'a Organization,
    pub role: &'a Role,
    pub content: Markup,
}

pub fn layout<'a>(content: &Content<'a>) -> Markup {
    html!(
        (DOCTYPE)
        html {
            head { (head(content.title, content.org)) }
            body x-data="{ open: false }" ":class"="{ open }"  {
                (header(content.title, content.role))
                (aside(content.path))
                main {
                    (&content.content)
                }
            }
        }
    )
}

pub fn header(title: &str, role: &Role) -> Markup {
    html!(
        header {
            .start {
                button.toggle "@click"="open = !open" {
                    template x-if="!open" {
                        i.fa-solid.fa-bars {}
                    }
                    template x-if="open" {
                        i.fa-solid.fa-xmark {}
                    }
                }
                .title {
                    h3 { (title) }
                }
            }
            .end {
                .account {
                    span { (format!("{} ({})", role.user_name, role.user_email)) }
                }
            }
        }
    )
}

pub fn head(title: &str, org: &Organization) -> Markup {
    html!(
        meta charset="UTF-8";
        meta name="viewport" content="width=device-width, initial-scale=1.0";
        title { (title) " - " (&org.title) }
        link rel="icon" href="/assets/dynamic/favicon.ico" {}
        link rel="preconnect" href="https://fonts.googleapis.com" {}
        link rel="preconnect" href="https://fonts.gstatic.com" crossorigin {}
        link href="https://fonts.googleapis.com/css2?family=Inter:ital,opsz,wght@0,14..32,100..900;1,14..32,100..900&display=swap" rel="stylesheet"{}
        link rel="stylesheet" href="/assets/static/dashboard/dashboard.min.css"{}

        script src="https://kit.fontawesome.com/952b0b64e9.js" crossorigin="anonymous"{}
        script type="module" src="/assets/static/dashboard/dashboard.min.js" {}
    )
}

pub fn aside(path: &str) -> Markup {
    html!(
        aside {
            nav {
                ul {
                    (link(path, "/dashboard", "Dashboard", "fa-house"))
                    (link(path, "/dashboard/pages", "Sitios", "fa-sitemap"))
                    (link(path, "/dashboard/events", "Eventos", "fa-calendar"))
                    (link(path, "/dashboard/products", "Productos", "fa-box"))
                    (link(path, "/dashboard/roles", "Roles", "fa-users"))
                    (link(path, "/dashboard/integrations", "Integraciones", "fa-plug"))
                }
            }
            .footer{
                ul {
                    li {
                        form action="/auth/logout" method="POST" x-ref="logout" {
                            a "@click.prevent"="$refs.logout.submit()" {
                                i.fa-solid.fa-arrow-right-from-bracket {}
                                span { "Cerrar sesión" }
                            }
                        }
                    }
                }
            }
        }
    )
}

pub fn link(suffix: &str, path: &str, text: &str, icon: &str) -> Markup {
    html!(
        li.active[path.ends_with(suffix)] title=(text) {
            a href=(path) hx-boost="true" {
                i .fa-solid .(icon) {}
                span { (text) }
            }
        }
    )
}

pub fn toast(message: &str, variant: Variant) -> Markup {
    html!(
        div hx-swap-oob="beforeend:body" {
            .toast.(variant) hx-trigger="click, load delay:5s" hx-get="/dashboard/empty" hx-swap="outerHTML swap:100ms" {
                span { (message) }
                button class="close" {
                    i class="fa-solid fa-xmark";
                }
            }
        }
    )
}

pub fn modal(title: &str, content: Markup) -> Markup {
    html!(
        dialog
            hx-trigger="click target:dialog, click from:.close, keyup[key == 'Escape'] from:window"
            hx-get="/dashboard/empty"
            hx-swap="outerHTML swap:100ms" {
            article {
                header {
                    strong class="title" { (title) }
                    button class="close" {
                        i class="fa-solid fa-xmark" {}
                    }
                }
                (content)
            }
        }
    )
}
