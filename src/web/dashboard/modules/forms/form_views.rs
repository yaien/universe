use std::collections::{HashMap, HashSet};

use maud::{Markup, html};
use serde_json::json;

use crate::app::forms::{Form, FormField, FormSubmission};
use crate::web::dashboard::modules::base::modal;

pub fn forms_list(forms: Vec<Form>) -> Markup {
    return html! {
        .container data-scope="forms" {
            .actions {
                button.clear hx-get="/dashboard/forms?fragment=create" hx-target=".container" hx-swap="beforeend" {
                    .fa-solid.fa-plus {}
                }
            }
            .list {
                @for form in forms {
                    a.item.detail href=(format!("/dashboard/forms/{}", form.id)) hx-boost="true" {
                       (form.name)
                    }
                }
            }
        }
    };
}

pub fn form_detail(form: Form) -> Markup {
    return html!(
        #form.container data-scope="form" {
            .forms {
                (base_form(&form))
                (field_list(&form))
            }
            .submissions {
                (form_submissions(&form))
            }
        }
    );
}

pub fn create_form_modal() -> Markup {
    return modal(
        "Crear Formulario",
        html! {
            form hx-post="/dashboard/forms" hx-target="dialog" hx-swap="outerHTML swap:250ms" hx-disable="find button"{
                fieldset {
                    legend { "Nombre" }
                    input name="name" required {}
                }
                fieldset {
                    legend { "Nombre Clave" }
                    input name="codename" required {}
                }
                .actions.text-center {
                    button type="submit" { "Crear" }
                }
            }
        },
    );
}

pub fn base_form(form: &Form) -> Markup {
    html! {
        article #base {
            .body {
                h4 { "Formulario" }
                form {
                    fieldset {
                        label { "Nombre" }
                        input name="name" value=(form.name) required {}
                    }
                    fieldset {
                        label { "Nombre Clave" }
                        input name="codename" value=(form.codename) required {}
                    }
                    .actions {
                        button.danger type="button" hx-delete=(format!("/dashboard/forms/{}?fragment=delete", form.id)) hx-target="#form" hx-swap="beforeend" {
                            "Eliminar"
                        }
                        button type="submit" { "Guardar" }
                    }
                }
            }
        }
    }
}

pub fn field_list(form: &Form) -> Markup {
    html! {
        article #fields {
            .body {
                h4 { "Campos" }
                .actions {
                    button.clear hx-get=(format!("/dashboard/forms/{}?fragment=create_field", form.id)) hx-target="#fields" hx-swap="outerHTML" {
                        i.fa-solid.fa-plus {}
                    }
                }
                ul.list {
                    @for field in &form.fields {
                        li.item.detail
                            hx-get=(format!("/dashboard/forms/{}?fragment=update_field&field_id={}", form.id, field.id))
                            hx-target="#fields"
                            hx-swap="outerHTML" {
                                (field.label)
                            }
                    }
                }
            }
        }
    }
}

pub fn create_field_form(form: &Form) -> Markup {
    html! {
        article #fields {
            .body {
                h4 { "Crear campo" }
                form hx-post=(format!("/dashboard/forms/{}/fields", form.id)) hx-target="#fields" hx-swap="outerHTML" {
                    fieldset {
                        label { "Etiqueta" }
                        input name="label" required {}
                    }
                    fieldset {
                        label { "Nombre" }
                        input name="name" required {}
                    }
                    .actions {
                        button hx-get=(format!("/dashboard/forms/{}?fragment=fields", form.id)) hx-target="#fields" hx-swap="outerHTML" {
                            "Volver"
                        }
                        button type="submit" { "Guardar" }
                    }
                }
            }
        }
    }
}

pub fn update_field_form(form: &Form, field: &FormField) -> Markup {
    html! {
        article #fields {
            .body {
                h4 { "Editar campo" }
                .actions {

                }
                form hx-put=(format!("/dashboard/forms/{}/fields/{}", form.id, field.id)) hx-target="#fields" hx-swap="outerHTML" {
                    fieldset {
                        label { "Etiqueta" }
                        input name="label" value=(field.label) required {}
                    }
                    fieldset {
                        label { "Nombre" }
                        input name="name" value=(field.name) required {}
                    }
                    .actions {
                        .actions {
                            button hx-get=(format!("/dashboard/forms/{}?fragment=fields", form.id)) hx-target="#fields" hx-swap="outerHTML" { "Volver" }
                        }
                        .actions {
                            button.danger type="button" hx-delete=(format!("/dashboard/forms/{}/fields/{}", form.id, field.id)) hx-target="#fields" hx-swap="outerHTML" { "Eliminar" }
                            button type="submit" { "Guardar" }
                        }
                    }
                }
            }
        }
    }
}

pub fn form_submissions(form: &Form) -> Markup {
    html! {
        article #submissions {
            .body {
                h4 { "Registros" }
                fieldset {
                    input type="search"
                        placeholder="Buscar..."
                        name="search"
                        hx-get=(format!("/dashboard/forms/{}/submissions", form.id))
                        hx-trigger="input changed delay:500ms"
                        hx-target="#results"
                        {}
                }
                table {
                    thead {
                        tr {
                            @for field in &form.fields {
                                th { (field.label) }
                            }
                        }
                    }
                    tbody #results hx-trigger="load" hx-get=(format!("/dashboard/forms/{}/submissions", form.id)) {

                    }
                }

            }
        }
    }
}

pub fn form_submission(
    form: &Form,
    submissions: &Vec<FormSubmission>,
    search: &Option<String>,
    next_offset: &Option<u32>,
    limit: &Option<u32>,
) -> Markup {
    html!(
        @for (index, submission) in submissions.iter().enumerate() {
            @let is_last = index == submissions.len() - 1;
            tr
                hx-get=[is_last.then_some(format!("/dashboard/forms/{}/submissions", form.id))]
                hx-trigger=[is_last.then_some("revealed")]
                hx-vals=[is_last.then_some(json!({"search": search, "offset": next_offset, "limit": limit }))]
                hx-swap=[is_last.then_some("after")] {

                @for field in &form.fields {
                    @if let Some(answer) = submission.answers.iter().find(|answer| answer.field_id == field.id) {
                            td { (answer.value) }
                    }
                }

            }
        }
    )
}
