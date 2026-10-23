use maud::html;

use crate::app::sitemaps::contents::{Query, Source};

use super::*;

pub fn vars(state: &ViewState) -> Markup {
    html!(
        .vars {
            .actions {
                button.clear hx-get="/dashboard/sitemaps"
                    hx-vals=(json!({ "section": Section::CreateVar }))
                    hx-target="#editor"
                    hx-swap="outerHTML" {
                    i.fa-solid.fa-plus {}
                }
            }

            ul.list {
                @if let Some(vars) = state.vars.as_ref() {
                    @for var in vars {
                        li.item.var.detail
                            hx-get="/dashboard/sitemaps"
                            hx-target="#editor"
                            hx-swap="outerHTML"
                            hx-vals=(json!({ "section": Section::EditVar, "var_id": var.id })) {

                            pre { (var.name) }
                            span { (var.query.to_string())}
                        }
                    }
                }
            }
        }
    )
}

pub fn create_var(state: &ViewState) -> Markup {
    html! {
        form hx-post="/dashboard/sitemaps/vars" hx-target="#section" autocomplete="off" x-data=(json!({ "queries": Query::descriptors(), "query": "" })) {
            fieldset {
                legend { "Nombre de variable" }
                input name="name" required {}
            }

            fieldset {
                legend { "Consulta" }
                select name="query" required x-model="query" {
                    template x-for="query in queries" {
                        option x-bind:value="query.name" x-text="query.label" {}
                    }
                }
            }

            template x-if="queries.find(q => q.name === query)?.args?.length > 0" {
                fieldset {
                    legend { "Fuente de carga de argumentos" }
                    select name="source" required x-model="source" {
                        @for source in Source::iter() {
                            option value=(source) { (source.to_string()) }
                        }
                    }
                }
            }



            #forms-fieldset {}

            .actions {
                button { "Crear" }
            }
        }
    }
}

pub fn edit_var(var: &Option<Var>) -> Markup {
    html! {
        @if let Some(var) = var {
            form hx-put=(format!("/dashboard/sitemaps/vars/{}", var.id))
                hx-target="#section"
                autocomplete="off"
                x-data=(json!({ "queries": Query::descriptors(), "query": var.query })) {

                fieldset {
                    legend { "Nombre de variable" }
                    input name="name" required value=(var.name) {}
                }

                fieldset {
                    legend { "Consulta" }
                    select name="query" required x-model="query" {
                        template x-for="query in queries" {
                            option x-bind:value="query.name" x-text="query.label" {}
                        }
                    }
                }

                template x-if="queries.find(q => q.name === query)?.args?.length > 0" {
                    fieldset {
                        legend { "Fuente de carga de argumentos" }
                        select name="source" required value=[var.source.as_ref()] {
                            @for source in Source::iter() {
                                option value=(source) { (source.to_string()) }
                            }
                        }
                    }
                }

                .actions {
                    button { "Guardar" }
                }
            }
        }
    }
}
