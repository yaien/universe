use super::*;

pub fn initial(state: &ViewState) -> Markup {
    let selected_is_page = (state.model_type == ModelType::Page).then_some("");
    let selected_is_layout = (state.model_type == ModelType::Layout).then_some("");
    let selected_is_email = (state.model_type == ModelType::Email).then_some("");

    html!(
        form autocomplete="off" {
            @if let Some(sitemaps) = &state.sitemaps {
                fieldset role="group" {
                    legend { "Mapa de Sitio"}
                    select
                        name="sitemap_branch"
                        autocomplete="off"
                        hx-get="/dashboard/sitemaps"
                        hx-target="#content"
                        hx-swap="outerHTML" {

                        @for sitemap in sitemaps {
                            option value=(sitemap.branch) selected=[(sitemap.id == state.sitemap.id).then_some("")] { (sitemap.branch) }
                        }
                    }

                }
            }
            fieldset role="group" {
                legend { "Tipo de Plantilla" }
                select
                    name="model_type"
                    autocomplete="off"
                    hx-get="/dashboard/sitemaps"
                    hx-target="#content"
                    hx-swap="outerHTML" {
                    option value="page" selected=[selected_is_page] { "Sitio" }
                    option value="layout" selected=[selected_is_layout] { "Diseño" }
                    option value="email" selected=[selected_is_email] { "Correo" }
                }
            }
            fieldset {
                legend { "Seleccionar Plantilla" }
                select name="model_id" hx-get="/dashboard/sitemaps" hx-target="#content" hx-swap="outerHTML" required  {
                    @match &state.model {
                        Some(Model::Page(selected)) => {
                            @if let Some(pages) = &state.pages {
                                @for page in pages {
                                    option value=(&page.id) selected=[(page.id == selected.id).then_some("true")] { (page.name) }
                                }
                            }
                        }
                        Some(Model::Layout(selected)) => {
                            @if let Some(layouts) = &state.layouts {
                                @for layout in layouts {
                                    option value=(&layout.id) selected=[(layout.id == selected.id).then_some("")] { (layout.name) }
                                }
                            }
                        }
                        Some(Model::Email(selected)) => {
                            @if let Some(emails) = &state.emails {
                                @for email in emails {
                                    option value=(&email.id) selected=[(email.id == selected.id).then_some("")] { (email.name) }
                                }
                            }
                        }
                        _=> {}

                    }
                }
            }
        }



        div role="group" {
            button title="Crear" hx-get="/dashboard/sitemaps" hx-target="#editor" hx-swap="outerHTML"  hx-vals=(json!({"section": Section::Create}))  {
                i.fa-solid.fa-plus {}
            }

            button title="Eliminar" hx-get="/dashboard/sitemaps" hx-target="#editor" hx-swap="outerHTML"  hx-vals=(json!({"section": Section::Delete}))  {
                i.fa-solid.fa-trash {}
            }

            button title="Publicar" hx-get="/dashboard/sitemaps" hx-target="#editor" hx-swap="outerHTML"  hx-vals=(json!({"section": Section::Publish}))  {
                i.fa-solid.fa-upload {}
            }
        }

    )
}

pub fn create(state: &ViewState) -> Markup {
    html!(
        .actions {
            button
                type="button"
                hx-trigger="click, deleted from:body, renamed from:body"
                hx-get="/dashboard/sitemaps"
                hx-vals=(json!({ "section": Section::Initial}))
                hx-target="#editor"
                hx-swap="outerHTML"
            {
                "Volver"
            }
        }
        fieldset x-data="{ modelType: 'page', urls: { page: '/dashboard/sitemaps/pages', layout: '/dashboard/sitemaps/layouts' } }" {
            legend { "Agregar Modelo" }
            form x-bind:hx-post="urls[modelType]" hx-target="#section" {
                fieldset role="group" {
                    legend {"Tipo de Plantilla"}
                    select name="model_type" x-model="modelType" {
                        option value="page" { "Sitio" }
                        option value="layout" {"Diseño"}
                    }
                }

                template x-if="modelType === 'page'" {
                    div {
                        fieldset {
                            legend { "Path" }
                            .group {
                                span { (state.organization.url)"/" }
                                input name="path" {}
                            }
                        }
                        fieldset {
                            legend { "Nombre" }
                            input name="name" required {}
                        }
                        fieldset {
                            legend { "Titulo" }
                            input name="title" required {}
                        }
                    }
                }

                template x-if="modelType === 'layout'" {
                    div {
                        fieldset {
                            legend { "Nombre" }
                            input name="name" required {}
                        }
                    }
                }

                .actions {
                    button {"Crear"}
                }
            }
        }
        fieldset {
            legend { "Sincronizar Mapa de Sitio" }
            form hx-post="/dashboard/sitemaps/branches" hx-target="#content" hx-swap="outerHTML" {
                fieldset {
                    legend { "Nombre" }
                    .group {
                        span { "draft/"}
                        input name="name" required {}
                    }
                }

                .actions {
                    button {"Sincronizar"}
                }
            }
        }
    )
}

pub fn delete(state: &ViewState) -> Markup {
    html!(
        .actions {
            button
                type="button"
                hx-trigger="click, deleted from:body, renamed from:body"
                hx-get="/dashboard/sitemaps"
                hx-vals=(json!({ "section": Section::Initial}))
                hx-target="#editor"
                hx-swap="outerHTML"
            {
                "Volver"
            }
        }
        .delete {
            @let count = match &state.model_type {
                ModelType::Page => state.pages.as_ref().map(|p| p.len()).unwrap_or(0 as usize),
                ModelType::Layout => state.layouts.as_ref().map(|l| l.len()).unwrap_or(0 as usize),
                _ => 0,
            };

            @if count > 1 {
                fieldset {
                    legend { "Eliminar plantilla" }
                    p {
                        @match &state.model {
                            Some(Model::Page(page)) => {
                                "Estás seguro de que deseas eliminar la página " b { (page.name) } "?"
                            },
                            Some(Model::Layout(layout)) => {
                                "Estás seguro de que deseas eliminar el layout " b { (layout.name) } "?"
                            },
                            Some(Model::Email(email)) => {
                                "Estás seguro de que deseas eliminar el email " b { (email.name) } "?"
                            },
                            _ => ""
                        }
                    }
                    .actions {
                        @let action = match &state.model {
                            Some(Model::Page(page)) => format!("/dashboard/sitemaps/pages/{}", page.id),
                            Some(Model::Layout(layout)) => format!("/dashboard/sitemaps/layouts/{}", layout.id),
                            _ => "".into(),
                        };
                        button.danger hx-delete=(action) hx-target="#content" hx-swap="outerHTML" {
                            "Eliminar"
                        }
                    }
                }
            }
            @if let Some(sitemaps) = &state.sitemaps {
                @if sitemaps.len() > 1 && state.sitemap.branch != Branch::DRAFT {
                    fieldset {
                        legend { "Eliminar mapa de sitio" }
                        p {
                            "Estás seguro de que deseas eliminar el mapa de sitio " b { (state.sitemap.branch) } "?"
                        }
                        .actions {
                            button.danger hx-post="/dashboard/sitemaps" hx-target="#content" hx-swap="outerHTML" hx-vals=(json!({ "action": "delete_sitemap"})) {
                                "Eliminar"
                            }
                        }
                    }
                }
            }
        }
    )
}

pub fn publish() -> Markup {
    html!(
        .actions {
            button
                type="button"
                hx-trigger="click, deleted from:body, renamed from:body"
                hx-get="/dashboard/sitemaps"
                hx-vals=(json!({ "section": Section::Initial}))
                hx-target="#editor"
                hx-swap="outerHTML"
            {
                "Volver"
            }
        }
        .publish {
            p {
                "Al publicar el mapa de sitio actual, este estará disponible para los usuarios finales"
            }
            .actions {
                button.warning hx-post="/dashboard/sitemaps/publish" hx-swap="none" { "Publicar" }
            }
        }
    )
}
