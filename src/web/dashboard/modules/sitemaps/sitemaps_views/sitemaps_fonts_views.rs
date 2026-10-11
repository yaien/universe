use super::*;

pub fn fonts(sitemap_fonts: &Option<Vec<SitemapFont>>) -> Markup {
    html! (
        @if let Some(sitemap_fonts) = sitemap_fonts {
            .fonts {
                .list {
                    @for sitemap_font in sitemap_fonts {
                        .font
                            title=(sitemap_font.family)
                            hx-get="/dashboard/sitemaps"
                            hx-target="#editor"
                            hx-swap="outerHTML"
                            hx-vals=(json!({ "section": Section::BrowseFonts, "sitemap_font_id": sitemap_font.id }))
                        {
                            .preview
                                x-data=(
                                    format!("font({{ family: {:?}, url: {:?} }})",
                                    sitemap_font.family, sitemap_font.files["regular"]
                                ))
                                ":style"="style"
                            {
                                (sitemap_font.family)
                            }
                            span { (sitemap_font.tag) }
                        }
                    }

                }
                @if sitemap_fonts.is_empty() {
                    p style="text-align: center" {
                        "Aún no hay fuentes configuradas."
                        br;
                        "Agrega una nueva fuente para comenzar"
                    }
                }
                .actions {
                    button
                        hx-get="/dashboard/sitemaps"
                        hx-target="#editor"
                        hx-swap="outerHTML"
                        hx-vals=(json!({ "section": Section::BrowseFonts }))
                    {
                        "Fuentes"
                    }
                }
            }
        }
    )
}

pub fn browse_fonts(state: &ViewState) -> Markup {
    html! {
        .fonts {
            .actions {
                button
                    hx-get="/dashboard/sitemaps"
                    hx-target="#editor"
                    hx-vals=(json!({ "section": Section::Fonts }))
                    hx-swap="outerHTML"
                {
                    "Volver"
                }
            }
            fieldset role="group" {
                legend { "Fuentes" }
                input
                    type="search"
                    name="browsed_fonts_query"
                    placeholder="Buscar fuente"
                    hx-trigger="input changed delay:500ms"
                    hx-get="/dashboard/sitemaps"
                    hx-target="#browsed-fonts"
                    hx-indicator=".fonts"
                    {}
            }

            #browsed-fonts.scrollable hx-vals:inherited=(json!({ "section": Section::ConfigureFont })) {
                (browse_fonts_list(&state.browsed_fonts, &state.browsed_font_query, &state.browsed_font_limit, &state.browsed_font_offset))
            }
            .spinner.htmx-indicator {
                i.fa-solid.fa-spinner {}
            }
        }
    }
}

pub fn browse_fonts_list(
    fonts: &Option<Vec<Font>>,
    query: &Option<String>,
    limit: &Option<u16>,
    offset: &Option<u16>,
) -> Markup {
    html!(
        @if let Some(fonts) = fonts {
            @for (index, font) in fonts.iter().enumerate() {
                @let is_last = index == fonts.len() - 1;
                .font
                    hx-get=[is_last.then_some("/dashboard/sitemaps")]
                    hx-trigger=[is_last.then_some("intersect once")]
                    hx-swap=[is_last.then_some("beforeend")]
                    hx-indicator=[is_last.then_some(".fonts")]
                    hx-target=[is_last.then_some("#browsed-fonts")]
                    hx-vals=[is_last.then_some(json!({
                            "browsed_fonts_query": query.clone().unwrap_or("".into()),
                            "browsed_fonts_limit": limit.unwrap_or(10),
                            "browsed_fonts_offset": offset.unwrap_or(0) + limit.unwrap_or(10)
                    }))]
                {
                    div
                        hx-get="/dashboard/sitemaps"
                        hx-target="#editor"
                        hx-swap="outerHTML"
                        hx-vals:append=(json!({ "browsed_font_id": font.id }))
                    {


                        @if let Some(file) = font.files.get("regular").or_else(|| font.files.values().next()) {
                            .preview
                                x-data=(format!{"font({{ family: {:?}, url: {:?} }})", font.family, file})
                                ":style"="style"
                            {
                                (font.family)
                            }
                        }
                    }
                }
            }
        }
    )
}

pub fn configure_font(state: &ViewState) -> Markup {
    html! (
        @if let Some(browsed_font) = &state.browsed_font {
            .fonts {
                .configure {
                    div class="actions" {
                        button
                            hx-trigger="click, updated from:body"
                            hx-get="/dashboard/sitemaps"
                            hx-target="#editor"
                            hx-swap="outerHTML"
                            hx-vals=(json!({ "section": Section::BrowseFonts }))
                        {
                            "Volver"
                        }
                    }
                    h1.preview
                        ":style"="style"
                        x-data=(format!(
                            "font({{ family: {:?}, url: {:?} }})",
                            browsed_font.family, browsed_font.files["regular"]
                        ))
                    {
                        (browsed_font.family)
                    }
                    form
                        hx-post=[state.sitemap_font.is_none().then_some("/dashboard/sitemaps/fonts")]
                        hx-put=[state.sitemap_font.as_ref().map(|f| format!("/dashboard/sitemaps/fonts/{}", f.id))]
                        hx-target=".fonts" hx-swap="outerHTML" {
                        small {

                            "Usa \"primary\"  para cambiar la fuente base de la página o \"headings\" para cambiar los
                            encabezados"
                        }

                        @let sitemap_font_name = match &state.sitemap_font {
                            Some(sitemap_font) => Some(&sitemap_font.tag),
                            None => None
                        };
                        fieldset {
                            legend { "Tag" }
                            input name="tag" autocomplete="off" required value=[sitemap_font_name] {}
                        }

                        input name="font_id" value=[state.browsed_font.as_ref().map(|f| f.id)] hidden {}

                        div class="actions" {
                            button type="submit" { "Guardar" }
                        }
                    }
                }
            }
        }
    )
}
