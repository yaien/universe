use super::*;

pub fn files(state: &ViewState) -> Markup {
    html!(
        .grow.files {
            .actions x-data="progress"{
                form
                    hx-trigger="change from:input changed"
                    hx-post="/dashboard/sitemaps/files"
                    hx-target="#files"
                    hx-encoding="multipart/form-data"
                    "@htmx:xhr:progress"="progress($event)"
                {
                    input type="file" x-ref="input" hidden name="files" accept="image/png,image/jpeg,image/jpg,video/mp4,audio/mp3" multiple {}
                    button type="button" "@click"="$refs.input.click()" class="clear" {
                        i.fa-solid.fa-plus {}
                    }
                    template x-if="loading" {
                        .progress {
                            div ":style"="{ width: percent + '%' }" {}
                        }
                    }
                }
            }
            #files .grid {
                (file_grid(&state))
            }
        }
    )
}

pub fn file_grid(state: &ViewState) -> Markup {
    let Some(files) = &state.files else {
        return html!();
    };

    html!(
        @for file in files {
            .item id=(file.id)  {
                div x-data {
                    .hover
                        title=(file.name)
                        hx-get="/dashboard/sitemaps"
                        hx-target="#editor"
                        hx-swap="outerHTML"
                        hx-vals=(json!({ "file_id": file.id, "section": Section::File }))
                        "@mouseenter"="$refs.player?.play()"
                        "@mouseleave"="$refs.player?.pause()"  {}

                    @if file.preset == "image" {
                        (file_image_setup(&file.id, state, true))
                        img src=(format!("/assets/dynamic/files/{}", file.name)) title=(file.name) alt=(file.name) {}
                    }
                    @if file.preset == "video" {
                        video x-ref="player" src=(format!("/assets/dynamic/files/{}", file.name)) title=(file.name) alt=(file.name) muted {}
                    }

                    @if file.preset == "audio" {
                        .setup.small role="group" {
                            button "@click"="$refs.audio?.play()" {
                                i.fa-solid.fa-play  {}
                            }
                        }
                        .audio.small {
                            i.fa-solid.fa-music {}
                            audio x-ref="audio" src=(format!("/assets/dynamic/files/{}", file.name)) title=(file.name) alt=(file.name) {}
                        }
                    }

                }
            }
        }
    )
}

pub fn file(state: &ViewState) -> Markup {
    let Some(file) = &state.file else {
        return html! {};
    };

    html!(
        .grow.files {
            #edit.edit {
                .actions {
                    button
                        type="button"
                        hx-trigger="click, deleted from:body, renamed from:body"
                        hx-get="/dashboard/sitemaps"
                        hx-vals=(json!({ "section": Section::Files }))
                        hx-target="#editor"
                        hx-swap="outerHTML"
                    {
                        "Volver"
                    }
                }
                .preview {
                    @if file.preset == "image" {
                        (file_image_setup(&file.id, state, false))
                        img src=(format!("/assets/dynamic/files/{}", file.name)) title=(file.name) alt=(file.name) {}
                    }
                    @if file.preset == "video" {
                        video src=(format!("/assets/dynamic/files/{}", file.name)) title=(file.name) alt=(file.name) controls {}
                    }

                    @if file.preset == "audio" {
                        .audio {
                            i.fa-solid.fa-music {}
                            audio src=(format!("/assets/dynamic/files/{}", file.name)) title=(file.name) alt=(file.name) controls {}
                        }
                    }
                }
                table.compact {
                    thead {
                        tr {
                            th { "Variante" }
                            th { "Dimensiones" }
                            th { "Tamaño" }
                            th { "Tipo" }
                        }
                    }
                    tbody {
                        @for format in file.formats.iter() {
                            tr {
                                td { (format.variant) }
                                td { (format.width) "x" (format.height) }
                                    td x-data=(format!("filesize({})", format.size)) x-text="size" {}
                                td { (format.content_type) }
                            }
                        }
                    }
                }
                form hx-put=(format!("/dashboard/sitemaps/files/{}", file.id)) hx-swap="none" {

                    fieldset {
                        legend { "Nombre" }
                        input name="name" value=(file.name) required {}
                    }
                    .actions.around {
                        button.danger type="button" hx-delete=(format!("/dashboard/sitemaps/files/{}", file.id)) hx-target="#edit" hx-swap="outerHTML" {
                            "Eliminar"
                        }
                        button type="submit" { "Guardar" }
                    }
                }
            }
        }
    )
}

macro_rules! file_active_button {
    ($func:ident, $icon:expr, $title:expr) => {
        pub fn $func(noclick: bool) -> Markup {
            html!(
                button.noclick[noclick] title=($title) {
                    i.($icon) {}
                }
            )
        }
    };
}

file_active_button!(
    file_og_image_active_button,
    "fa-solid fa-link",
    "Imagen de la página"
);

file_active_button!(
    file_favicon_active_button,
    "fa-solid fa-globe",
    "Icono de pestaña"
);

pub fn file_image_setup(file_id: &Id, state: &ViewState, small_mode: bool) -> Markup {
    html!(
        .setup.small[small_mode] role="group" {
            @if Some(*file_id) == state.sitemap.favicon_file_id {
                (file_favicon_active_button(small_mode))
            } @else if !small_mode {
                button.secondary
                    hx-patch="/dashboard/sitemaps/favicon"
                    hx-vals=(json!({ "file_id": file_id }))
                    hx-swap="outerHTML" {
                    i.fa-solid.fa-globe {}
                }
            }

            @if let Some(Model::Page(page)) = &state.model {
                @if Some(*file_id) == page.og_image_file_id {
                    (file_og_image_active_button(small_mode))
                } @else if !small_mode {
                    button.secondary
                        hx-patch="/dashboard/sitemaps/og_image"
                        hx-vals=(json!({ "file_id": file_id }))
                        hx-swap="outerHTML" {
                        i.fa-solid.fa-link {}
                    }
                }
            }
        }
    )
}
