use maud::{Markup, html};

use crate::app::auth::User;

pub fn users_list(users: &[User]) -> Markup {
    html! {
        .container data-scope="users" {
            article {
                .body {
                    fieldset {
                        input type="search"
                            placeholder="Buscar..."
                            name="search"
                            hx-get="/dashboard/users"
                            hx-trigger="input changed delay:500ms"
                            hx-target="#results"
                            {}
                    }

                    table {
                        thead {
                            tr {
                                th { "Nombre" }
                                th { "Email" }
                            }
                        }
                        tbody {
                            @for user in users {
                                tr {
                                    td { (user.name) }
                                    td { (user.email) }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
