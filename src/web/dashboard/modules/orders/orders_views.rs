use chrono::TimeZone;
use maud::{Markup, html};

use crate::app::store::Order;

pub fn order_list(orders: &[Order]) -> Markup {
    html!(
        .container data-scope="orders" {
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
                                th { "Creado" }
                                th { "Actualizado" }
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
                td { (order.amount) }
                td { (order.created_at) }
                td { (order.updated_at) }
                td { (order.status) }
                td { a.clear role="button" hx-boost="true" href=(format!("/dashboard/orders/{}", order.id)) { "Ver" } }
            }
        }
    )
}
