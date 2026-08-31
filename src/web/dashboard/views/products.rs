use maud::{Markup, html};

use crate::app::{Content, Presentation, Product, Products};
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
                            @if let Some(content) = product.presentations.first().map(|p| p.contents.first()).flatten() {
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

pub fn product_detail(product: &Product, presentation: &Option<Presentation>) -> Markup {
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
                            input type="checkbox" name="published" checked=(product.published) {}
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
            (pictures(product, presentation, &None, false))
        }
    )
}

pub fn presentations(product: &Product, presentation: &Option<Presentation>) -> Markup {
    html!(
        article #presentations {
            h4 { "Presentaciones" }
            form {
                div role="group" {
                    @for p in &product.presentations {
                        @if presentation.as_ref().map_or(false, |p| p.id == p.id) {
                            button.draggable.active
                                hx-get=(format!("/dashboard/products/{}?presentation={}", product.id, p.id))
                                hx-target="#presentations"
                                hx-swap="outerHTML"
                                hx-push-url="true" {
                                (p.name)
                            }
                        } @else {
                            button.draggable {
                                (p.name)
                            }
                        }
                    }
                    button.icon hx-post=(format!("/dashboard/products/{}/presentations", product.id)) hx-target="#presentations" hx-swap="outerHTML" {
                        .fa-solid.fa-plus {}
                    }
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
    presentation: &Option<Presentation>,
    content: &Option<Content>,
    swapobb: bool,
) -> Markup {
    html!(
        article #pictures .pictures hx-swap-oob=[swapobb.then_some("")] {
            @if let Some(presentation) = presentation {
                (pictures_section(product, presentation, content))
            } @else {
                (pictures_placeholder_section())
            }
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
    _product: &Product,
    presentation: &Presentation,
    content: &Option<Content>,
) -> Markup {
    let selected = content.as_ref().or(presentation.contents.first());

    html!(
        .pictures {
            .placeholder.big {
                @if let Some(selected) = selected {
                    img src=(format!("/assets/dynamic/files/{}", selected.file_id)) {}
                    .actions {
                        button.danger {
                            i.fa-regular.fa-trash-can {}
                        }
                    }
                }
            }
            .flex {
                form.flex {
                    @for content in &presentation.contents {
                        .placeholder.small.draggable {
                            img src=(format!("/assets/dynamic/files/{}", content.file_id)) {}
                        }
                    }
                }
                @for _ in 0..(Products::MAX_CONTENTS_PER_PRESENTATION - presentation.contents.len()) {
                    .placeholder.small {}
                }
                @if presentation.contents.len() < Products::MAX_CONTENTS_PER_PRESENTATION {
                    .placeholder.small {
                        .progress {
                            div {}
                        }
                    }
                    form {
                        input type="file" name="file" hidden="true" accept="image/*" {}
                        button.clear {
                            .fa-solid.fa-plus {}
                        }
                    }
                }
            }
        }
    )
}
