use crate::app::sitemaps::contents::Tool;

use super::*;

pub fn actions(state: &ViewState) -> Markup {
    html! {
        .template-actions {
            .actions {
                button.clear hx-get="/dashboard/sitemaps"
                    hx-vals=(json!({ "section": Section::CreateAction }))
                    hx-target="#editor"
                    hx-swap="outerHTML" {
                    i.fa-solid.fa-plus {}
                }
            }
            ul.list {
                @if let Some(actions) = &state.actions {
                    @for action in actions {
                        li.item.detail
                            hx-get="/dashboard/sitemaps"
                            hx-vals=(json!({ "section": Section::EditAction, "action_id": action.id }))
                            hx-target="#editor"
                            hx-swap="outerHTML" {
                            (action.name)
                        }
                    }
                }
            }
        }
    }
}

pub fn create_action() -> Markup {
    html!(
        .template-actions {
            form hx-post="/dashboard/sitemaps/actions" hx-target="#section" autocomplete="off" {
                fieldset {
                    legend { "Nombre" }
                    input name="name" required {}
                }
                fieldset {
                    legend { "Nombre Clave"}
                    input name="codename" required {}
                }

                fieldset {
                    legend { "Función" }
                    select name="tool" required hx-trigger="change, load" hx-get="/dashboard/sitemaps/actions/forms" hx-target="#forms-fieldset" hx-include="#form_id" {
                        @for tool in Tool::iter() {
                            option value=(tool) {
                                (tool.label())
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
    )
}

pub fn form_fieldset(forms: &[Form], selected_form_id: Option<Id>) -> Markup {
    html! {
        fieldset {
            legend { "Formulario" }
            select id="form_id" name="form_id" required {
                @for form in forms {
                    @let selected = selected_form_id.filter(|id| *id == form.id).map(|_| "");
                    option value=(form.id) selected=[selected] {
                        (form.name)
                    }
                }
            }
        }
    }
}

pub fn edit_action(action: &Option<Action>) -> Markup {
    html! {
        @if let Some(action) = &action {
            .template-actions {
                .actions {
                    button
                        hx-get="/dashboard/sitemaps"
                        hx-target="#editor"
                        hx-swap="outerHTML"
                        hx-vals=(json!({ "section": Section::Actions })) {
                            "Volver"
                        }
                }
                form hx-put=(format!("/dashboard/sitemaps/actions/{}", action.id)) hx-swap="none" autocomplete="off" {
                    fieldset {
                        legend { "Nombre" }
                        input name="name" value=(action.name)  required {}
                    }
                    fieldset {
                        legend { "Nombre Clave"}
                        input name="codename" value=(action.codename) required {}
                    }
                    fieldset {
                        legend { "Función" }
                        select name="tool" required hx-trigger="change, load" hx-get="/dashboard/sitemaps/actions/forms" hx-target="#forms-fieldset" hx-include="#form_id" {
                            @for tool in Tool::iter() {
                                option value=(tool) selected=[(tool == action.tool).then_some("")] {
                                    (tool.label())
                                }
                            }
                        }
                    }

                    #forms-fieldset {}

                    fieldset.code {
                        legend { "Cuerpo de Respuesta"}
                        .monaco x-data=(format!(r#"monaco({{ language: "html", source: {:?} }})"#, action.response_body_template)) {
                            .spinner x-show="loading" {
                                i class="fa-solid fa-spinner" {}
                            }
                            input name="response_body_template" ":value"="source" hidden {}
                        }
                    }
                    .actions {
                        button { "Guardar" }
                    }
                }
            }
        }
    }
}
