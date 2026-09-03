use actix_multipart::form::json;
use maud::{Markup, html};
use serde_json::json;

use crate::app::store::{
    Content, MAX_CONTENTS_PER_PRESENTATION, Presentation, Presentations, Product,
};
use crate::web::dashboard::views;

pub fn product_list(products: Vec<Product>) -> Markup {
    html!(
        .container data-scope="products" {
            .actions {
                button.clear hx-get="/dashboard/products?fragment=create" hx-target=".container" hx-swap="beforeend" {
                    .fa-solid.fa-plus {}
                }
            }
            .products {
                @for product in products {
                    a.detail role="article" href=(format!("/dashboard/products/{}", product.id)) hx-boost="true" {
                        header {
                            @if let Some(content) = product.presentations.first().and_then(|p| p.contents.first()) {
                                img src=(format!("/assets/dynamic/files/{}", content.file_id)){}
                            } @else {
                                .placeholder {}
                            }
                        }
                        div {
                            h4 { (product.name) }
                        }
                    }
                }
            }
        }
    )
}

pub fn create_modal() -> Markup {
    views::layout::modal(
        "Crear Producto",
        html!(
            form hx-post="/dashboard/products" hx-target="dialog" hx-swap="outerHTML swap:250ms" hx-disable="find button"{
                fieldset {
                    legend { "Nombre" }
                    input name="name" required {}
                }
                .actions.text-center {
                    button type="submit" { "Crear" }
                }
            }
        ),
    )
}

pub fn delete_modal(product: &Product) -> Markup {
    views::layout::modal(
        "Eliminar Producto",
        html!(
            form hx-delete=(format!("/dashboard/products/{}", product.id)) hx-target="dialog" hx-swap="outerHTML swap:250ms" hx-disable="find button" {
                p {
                    center { "Estas seguro de eliminar el producto" strong { (product.name) } "?" }
                }
                .actions.text-center {
                    button type="submit" { "Eliminar" }
                }
            }
        ),
    )
}

pub fn product_detail(product: &Product, presentation: &Option<&Presentation>) -> Markup {
    html!(
        #product data-scope="product" {
            .forms {
                article {
                    h4 { "Producto" }
                    form hx-put=(format!("/dashboard/products/{}", product.id)) hx-target="dialog" hx-swap="outerHTML swap:250ms" hx-disable="find button" {
                        fieldset {
                            legend { "Nombre" }
                            input name="name" value=(product.name) required {}
                        }
                        fieldset {
                            legend { "Publicado" }
                            select name="published" {
                                option value="true" selected=[product.published.then_some("")] { "Sí" }
                                option value="false" selected=[(!product.published).then_some("")] { "No" }
                            }
                        }
                        .actions {
                            button.danger type="button" hx-get=(format!("/dashboard/products/{}/delete", product.id)) hx-target="#product" hx-swap="beforeend" {
                                "Eliminar"
                            }
                            button type="submit" { "Guardar" }
                        }
                    }
                }
                (presentations(product, presentation))
            }
            (pictures(product, presentation, &None))
        }
    )
}

pub fn presentations(product: &Product, presentation: &Option<&Presentation>) -> Markup {
    html!(
        article #presentations {
            h4 { "Presentaciones" }
            div role="group" {
                @for p in &product.presentations {
                    @if presentation.as_ref().map_or(false, |selected| selected.id == p.id) {
                        button.draggable.active { (p.name) }
                    } @else {
                        button.draggable
                            hx-get=(format!("/dashboard/products/{}?presentation_id={}", product.id, p.id))
                            hx-target="#presentations"
                            hx-swap="outerHTML"
                            hx-push-url="true" {
                            (p.name)
                        }
                    }
                }
                button.icon
                    hx-post=(format!("/dashboard/products/{}", product.id))
                    hx-target="#presentations"
                    hx-swap="outerHTML"
                    hx-vals=(json!({ "action": "create_presentation" })) {
                    .fa-solid.fa-plus {}
                }
            }
            @if let Some(presentation) = presentation {
                (presentation_form(product, presentation))
            }
        }
    )
}

pub fn presentation_form(_product: &Product, presentation: &Presentation) -> Markup {
    html!(
        form {
            fieldset {
                legend { "Nombre" }
                input name="name" value=(presentation.name) {}
            }
            fieldset {
                legend { "Inventario" }
                input name="quantity" type="number" min="0" value=(presentation.quantity) {}
            }
            fieldset {
                legend { "Precio" }
                input name="price" type="number" min="0" value=(presentation.price) {}
            }
            .actions {
                button.danger type="button" { "Eliminar" }
                button { "Guardar" }
            }

        }
    )
}

pub fn pictures(
    product: &Product,
    presentation: &Option<&Presentation>,
    content: &Option<&Content>,
) -> Markup {
    html!(
        article #pictures .pictures {
            @if let Some(presentation) = presentation {
                (pictures_section(product, presentation, content))
            } @else {
                (pictures_placeholder_section())
            }
        }
    )
}

pub fn pictures_partial(
    product: &Product,
    presentation: &Option<&Presentation>,
    content: &Option<&Content>,
) -> Markup {
    html!(
        hx-partial hx-target="#pictures" hx-swap="outerHTML" {
            (pictures(product, presentation, content))
        }
    )
}

pub fn pictures_placeholder_section() -> Markup {
    html!(
        .pictures {
            .placeholder.big {}
            .list {
                @for _ in 0..5 {
                    .placeholder.small {}
                }
            }
        }
    )
}

pub fn pictures_section(
    product: &Product,
    presentation: &Presentation,
    content: &Option<&Content>,
) -> Markup {
    let selected = content.or(presentation.contents.first());

    html!(
        .pictures {
            .placeholder.big {
                @if let Some(selected) = selected {
                    img src=(format!("/assets/dynamic/files/{}", selected.file_id)) {}
                    .actions {
                        button.danger
                            hx-post=(format!("/dashboard/products/{}", product.id))
                            hx-target="#pictures"
                            hx-swap="outerHTML"
                            hx-vals=(json!({ "action": "delete_content", "content_id": selected.id, "presentation_id": presentation.id })) {
                            i.fa-regular.fa-trash-can {}
                        }
                    }
                }
            }
            .flex {
                .flex
                    x-data="drag"
                    hx-post=(format!("/dashboard/products/{}", product.id))
                    hx-trigger="dragged"
                    hx-swap="none"
                    hx-include="find input" {

                    input type="hidden" name="action" value="sort_content" {}
                    input type="hidden" name="presentation_id" value=(presentation.id) {}
                    input type="hidden" name="toggled_content_id" x-bind:value="id" {}
                    input type="hidden" name="toggled_new_number" x-bind:value="number" {}


                    @for content in &presentation.contents {
                        #(content.id).placeholder.small.draggable
                            hx-get=(format!("/dashboard/products/{}?presentation_id={}&content_id={}", product.id, presentation.id, content.id))
                            hx-target="#pictures"
                            hx-swap="outerHTML" {
                            img src=(format!("/assets/dynamic/files/{}", content.file_id)) {}
                        }
                    }

                }
                @for _ in 0..(MAX_CONTENTS_PER_PRESENTATION - presentation.contents.len() as i64 - 1) {
                    .placeholder.small {}
                }
                @if MAX_CONTENTS_PER_PRESENTATION > presentation.contents.len() as i64 {
                    .placeholder.small {
                        form x-data
                            hx-post=(format!("/dashboard/products/{}/presentations/{}/contents", product.id, presentation.id))
                            hx-trigger="change from:input changed"
                            hx-target="#pictures"
                            hx-swap="outerHTML"
                            hx-encoding="multipart/form-data"
                            {
                            input type="file" name="files" hidden="true" accept="image/*" x-ref="file" multiple {}
                            button.clear type="button" "@click"="$refs.file.click()" {
                                .fa-solid.fa-plus {}
                            }
                        }
                    }
                }
            }
        }
    )
}
