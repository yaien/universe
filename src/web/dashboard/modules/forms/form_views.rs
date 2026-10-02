use maud::{Markup, html};

use crate::app::forms::Form;
use crate::web::dashboard::modules::base::modal;

pub fn forms_list(forms: Vec<Form>) -> Markup {
    return html! {
        .container data-scope="forms" {
            .actions {
                button.clear hx-get="/dashboard/forms?fragment=create" hx-target=".container" hx-swap="beforeend" {
                    .fa-solid.fa-plus {}
                }
            }
            ul.list {
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
    return html!();
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
