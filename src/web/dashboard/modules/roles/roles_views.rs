use maud::{Markup, html};

use crate::app::auth::Role;

pub fn role_list(roles: &[Role], current_role: &Role) -> Markup {
    html! {
        .container data-scope="roles" {
            article {
                .body {
                    table {
                        thead {
                            tr {
                                th { "Nombre" }
                                th { "Email" }
                                th { "Acciones" }
                            }
                        }
                        tbody {
                            @for role in roles {
                                tr {
                                    td { (role.user_name) }
                                    td { (role.user_email) }
                                    td {
                                        @if role.id != current_role.id && roles.len() > 1{
                                            button.danger {
                                                i.fa-solid.fa-trash {}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
