use chrono::TimeZone;
use maud::{Markup, html};

use crate::app::store::Order;

pub fn order_list(orders: &[Order]) -> Markup {
    html!(
        .container data-scope="orders" x-data="utils" {
            article {
                .body {
                    fieldset {
                        input type="search"
                            placeholder="Buscar..."
                            name="search"
                            hx-get="/dashboard/orders"
                            hx-trigger="input changed delay:500ms"
                            hx-target="#results"
                            {}
                    }

                    table {
                        thead {
                            tr {
                                th { "ID" }
                                th { "Nombre"}
                                th { "Correo"}
                                th { "Monto" }
                                th { "Fecha de Actualización" }
                                th { "Estado" }
                            }
                        }
                        tbody #results {
                            (order_list_items(orders))
                        }
                    }
                }
            }
        }
    )
}

pub fn order_list_items(orders: &[Order]) -> Markup {
    html!(
        @for order in orders {
            tr {
                td { (order.id) }
                td { (order.user_name) }
                td { (order.user_email) }
                td x-text=(format!("currency({})", order.amount)) { (order.amount) }
                td x-text=(format!("date('{}')", order.updated_at)) { (order.updated_at) }
                td { (order.status) }
                td { a.clear role="button" hx-boost="true" href=(format!("/dashboard/orders/{}", order.id)) { "Ver" } }
            }
        }
    )
}

pub fn order_details(order: Order) -> Markup {
    html!(
        .container data-scope="order" x-data="utils" {
            .info {
                article {
                    .body {
                        h3 { "Detalles de Orden" }
                        ul.list {
                            li.item {
                                span { "ID" }
                                span { (order.id) }
                            }
                            li.item {
                                span { "Nombre" }
                                span { (order.user_name) }
                            }
                            li.item {
                                span { "Email" }
                                span { (order.user_email) }
                            }
                            li.item {
                                span { "Monto" }
                                span x-text=(format!("currency({})", order.amount)) { (order.amount) }
                            }
                            li.item {
                                span { "Fecha de Creación" }
                                span x-text=(format!("date('{}')", order.created_at)) { (order.created_at) }
                            }
                            li.item {
                                span { "Fecha de Actualización" }
                                span x-text=(format!("date('{}')", order.updated_at)) { (order.updated_at) }
                            }
                            li.item {
                                span { "Estado" }
                                span { (order.status) }
                            }
                        }
                        .actions {
                            button { "Cancelar" }
                            button { "Aprobar"}
                        }
                    }
                }
                article {
                    .body {
                        h3 { "Información Adicional" }
                        ul {
                            @for field in order.form_fields {
                                li.item {
                                    span { (field.label) }
                                    @if let Some(answer) = order.form_submission_answers.iter().find(|a| a.field_id == field.id) {
                                        span { (answer.value) }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            .items {
                article {
                    .body {
                        h3 { "Items" }
                        ul.list {
                            @for item in order.items {
                                li.item {
                                    @if let Some(first_presentation_content_file_id) = item.first_presentation_content_file_id {
                                        img src=(format!("/assets/dynamic/files/{}", first_presentation_content_file_id)) alt="" {}
                                    } @else {
                                        .thumbnail {
                                            i.fa-solid.fa-image {}
                                        }
                                    }
                                    ul.list.compact.no-border.text {
                                        li.item {
                                            span { "Producto" }
                                            span {(item.product_name) }
                                        }
                                        li.item {
                                            span { "Presentación" }
                                            span {(item.presentation_name) }
                                        }
                                        li.item {
                                            span { "Precio Individual" }
                                            span {(item.price_per_unit) }
                                        }
                                        li.item {
                                            span { "Cantidad" }
                                            span {(item.quantity) }
                                        }
                                        li.item {
                                            span { "Precio Total" }
                                            span {(item.total_price) }
                                        }
                                    }
                                }
                            }
                            li.item {
                                strong { "Total" }
                                strong { (order.amount)}
                            }
                        }
                    }
                }
            }
        }
    )
}
