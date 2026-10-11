use maud::{Markup, html};
use serde_json::json;

use crate::app::forms::{Form, FormSubmission};

pub fn forms_list(forms: Vec<Form>) -> Markup {
    return html! {
        .container data-scope="forms" {
            @if forms.is_empty() {
                article {
                    .body {
                        p { "No hay formularios disponibles." }
                    }
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
            .submissions {
                (form_submissions(&form))
            }
        }
    );
}

pub fn form_submissions(form: &Form) -> Markup {
    html! {
        article #submissions {
            .body {
                h4 { (form.name) }
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
                    tbody #results hx-trigger="load" hx-get=(format!("/dashboard/forms/{}/submissions", form.id)) {}
                }


            }
        }
    }
}

pub fn form_submission(
    form: &Form,
    submissions: &Vec<FormSubmission>,
    search: &str,
    next_offset: &u32,
    limit: &u32,
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
