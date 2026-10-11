use super::*;

pub fn edit(model: &Option<Model>, org: &Organization, layouts: &Option<Vec<Layout>>) -> Markup {
    html!(
        @match model {
            Some(Model::Page(page)) => {
                form hx-put=(format!("/dashboard/sitemaps/pages/{}", page.id)) hx-swap="none" autocomplete="off" {
                    fieldset {
                        legend { "Nombre" }
                        input name="name" required value=(page.name) {}
                    }
                    fieldset role="group" {
                        legend { "Url" }
                        .group {
                            span { (org.url)"/" }
                            input name="path" value=(page.path) {}
                        }
                    }
                    fieldset {
                        legend { "Titulo" }
                        input name="title" value=(page.title) {}
                    }
                    fieldset {
                        legend { "Tipo" }
                        select name="og_type" {
                            @let options = [
                                ("", "Ninguno"),
                                ("website", "Sitio Web"),
                                ("article", "Artículo"),
                                ("profile", "Perfil"),
                                ("product", "Producto"),
                            ];

                            @for (value, label) in options {
                                option value=(value) selected=[(value == page.og_type).then_some("")] { (label) }
                            }
                        }
                    }
                    fieldset {
                        legend { "Descripción" }
                        textarea name="og_description" class="no-resize" rows="2" cols="10" {
                            (page.og_description)
                        }
                    }
                    fieldset role="group" {
                        legend { "Diseño" }
                        select name="layout_id" {
                            option value="" { "Ninguno" }
                            @if let Some(layouts) = layouts {
                                @for layout in layouts {
                                    option value=(layout.id) selected=[(Some(layout.id) == page.layout_id).then_some("")] { (layout.name) }
                                }
                            }
                        }
                    }
                    .actions {
                        button type="submit" { "Guardar" }
                    }
                }
            },
            Some(Model::Layout(layout)) => {
                form hx-put=(format!("/dashboard/sitemaps/layouts/{}", layout.id)) hx-swap="none" {
                    fieldset {
                        legend { "Nombre" }
                        input name="name" required value=(layout.name) {}
                    }
                    .actions {
                        button type="submit" { "Guardar" }
                    }
                }
            },
            Some(Model::Email(email)) => {
                form hx-put=(format!("/dashboard/sitemaps/emails/{}", email.id)) hx-swap="none" {
                    fieldset {
                        legend { "Asunto" }
                        textarea name="subject" class="no-resize" required cols="10" {
                            (email.subject)
                        }
                    }
                    .actions {
                        button type="submit" { "Guardar" }
                    }
                }
            },
            None => {}
        }
    )
}
