use crate::app::forms::FormField;

use super::*;

pub fn forms(forms: &Option<Vec<Form>>) -> Markup {
    html! {
        #forms .forms {
            .actions {
                button.clear hx-get="/dashboard/sitemaps"
                    hx-target="#editor"
                    hx-swap="outerHTML"
                    hx-vals=(json!({ "section": Section::CreateForm })) {
                    .fa-solid.fa-plus {}
                }
            }
            .list {
                @if let Some(forms) = forms {
                    @for form in forms {
                        .item.detail
                            hx-get="/dashboard/sitemaps"
                            hx-target="#editor"
                            hx-swap="outerHTML"
                            hx-vals=(json!({ "section": Section::EditForm, "form_id": form.id })) {
                            (form.name)
                        }
                    }
                }
            }
        }
    }
}

pub fn create_form() -> Markup {
    html! {
        .forms {
            form hx-post="/dashboard/sitemaps/forms" hx-target="dialog" hx-swap="outerHTML swap:250ms" hx-disable="find button"{
                fieldset {
                    legend { "Nombre" }
                    input name="name" required {}
                }
                .actions.text-center {
                    button hx-get="/dashboard/sitemaps" hx-target="#fields" hx-swap="outerHTML" hx-vals=(json!({ "section": Section::Forms })) {
                        "Volver"
                    }
                    button type="submit" { "Crear" }
                }
            }
        }
    }
}

pub fn edit_form(form: &Option<Form>) -> Markup {
    html! {
        @if let Some(form) = form {
            #forms .forms {
                .actions {
                    button
                        hx-trigger="click"
                        hx-get="/dashboard/sitemaps"
                        hx-vals=(json!({ "section": Section::Forms }))
                        hx-target="#editor"
                        hx-swap="outerHTML"
                    {
                        "Volver"
                    }
                }
                (base_form(form))
                (form_field_list(form))
            }
        }
    }
}

pub fn delete_form(form: &Form) -> Markup {
    html! {
        fieldset #base {
            legend { "Eliminar formulario" }
            form hx-delete=(format!("/dashboard/sitemaps/forms/{}", form.id)) hx-target="#section" hx-swap="outerHTML" {
                p {
                    center { "Estas seguro de eliminar el formulario " strong { (form.name) } "?" }
                }
                .actions.text-center {
                    button type="button" hx-get="/dashboard/sitemaps" hx-target="#base" hx-select="#base" { "Cancelar" }
                    button.danger type="submit" { "Eliminar" }
                }
            }
        }
    }
}

fn base_form(form: &Form) -> Markup {
    html! {
        fieldset #base {
            legend { "Formulario" }
            form hx-put=(format!("/dashboard/forms/{}", form.id)) hx-swap="none" {
                fieldset {
                    label { "Nombre" }
                    input name="name" value=(form.name) required {}
                }
                .actions {
                    button.danger type="button" hx-get=(format!("/dashboard/sitemaps/forms/{}/delete", form.id)) hx-target="#base" hx-swap="outerHTML" {
                        "Eliminar"
                    }
                    button type="submit" { "Guardar" }
                }
            }
        }
    }
}

pub fn form_field_list(form: &Form) -> Markup {
    html! {
        fieldset #fields {
            legend { "Campos" }
            .actions style="padding: 0" {
                button.clear
                    hx-get=(format!("/dashboard/sitemaps/forms/{}/fields/create", form.id))
                    hx-target="#fields"
                    hx-swap="outerHTML" {
                    i.fa-solid.fa-plus {}
                }
            }
            ul.list {
                @for field in &form.fields {
                    li.item.detail
                        hx-get=(format!("/dashboard/sitemaps/forms/{}/fields/{}", form.id, field.id))
                        hx-target="#fields"
                        hx-swap="outerHTML" {
                        (field.label)
                    }
                }
            }
        }
    }
}

pub fn create_field_form(form: &Form) -> Markup {
    html! {
        fieldset #fields {
            legend { "Crear campo" }
            form hx-post=(format!("/dashboard/sitemaps/forms/{}/fields", form.id)) hx-target="#fields" hx-swap="outerHTML" {
                fieldset {
                    legend { "Etiqueta" }
                    input name="label" required {}
                }
                fieldset {
                    legend { "Nombre" }
                    input name="name" required {}
                }
                .actions {
                    button hx-get="/dashboard/sitemaps" hx-select="#fields" hx-target="#fields" hx-swap="outerHTML" {
                        "Volver"
                    }
                    button type="submit" { "Guardar" }
                }
            }
        }
    }
}

pub fn update_field_form(form: &Form, field: &FormField) -> Markup {
    html! {
        fieldset #fields {
            legend { "Editar campo" }
            form hx-put=(format!("/dashboard/sitemaps/forms/{}/fields/{}", form.id, field.id)) hx-target="#fields" hx-swap="outerHTML" {
                fieldset {
                    legend { "Etiqueta" }
                    input name="label" value=(field.label) required {}
                }
                fieldset {
                    legend { "Nombre" }
                    input name="name" value=(field.name) required {}
                }
                .actions {
                    .actions {
                        button hx-get="/dashboard/sitemaps" hx-select="#fields" hx-target="#fields" hx-swap="outerHTML" {
                            "Volver"
                        }
                    }
                    .actions {
                        button.danger type="button" hx-delete=(format!("/dashboard/sitemaps/forms/{}/fields/{}", form.id, field.id)) hx-target="#fields" hx-swap="outerHTML" { "Eliminar" }
                        button type="submit" { "Guardar" }
                    }
                }
            }
        }
    }
}
