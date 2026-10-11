use super::*;

pub fn forms(forms: &Option<Vec<Form>>) -> Markup {
    html! {
        .forms {
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
                    button type="submit" { "Crear" }
                }
            }
        }
    }
}

pub fn edit_form(form: &Option<Form>) -> Markup {
    html! {
        @if let Some(form) = form {
            .forms {
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
                (field_list(form))
            }
        }
    }
}

fn base_form(form: &Form) -> Markup {
    html! {
        fieldset {
            legend { "Formulario" }
            form hx-put=(format!("/dashboard/forms/{}", form.id)) hx-swap="none" {
                fieldset {
                    label { "Nombre" }
                    input name="name" value=(form.name) required {}
                }
                .actions {
                    button.danger type="button" hx-get=(format!("/dashboard/forms/{}?fragment=delete", form.id)) hx-target="#form" hx-swap="beforeend" {
                        "Eliminar"
                    }
                    button type="submit" { "Guardar" }
                }
            }
        }
    }
}

fn field_list(form: &Form) -> Markup {
    html! {
        fieldset #fields {
            legend { "Campos" }
            .actions {
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
