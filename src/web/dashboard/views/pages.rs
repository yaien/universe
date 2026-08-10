use maud::{Markup, html};
use serde::{Deserialize, Serialize};
use serde_json::json;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::app::{Branch, Color, Email, File, Font, Layout, Page, Sitemap, SitemapFont};
use crate::infra::Id;

pub enum Model {
    Page(Page),
    Layout(Layout),
    Email(Email),
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ModelType {
    Page,
    Layout,
    Email,
}

#[derive(Serialize, Deserialize)]
pub struct SessionState {
    pub sitemap_branch: String,
    pub model_type: ModelType,
    pub model_id: Option<Id>,
    pub section: Section,
    pub file_id: Option<Id>,
    pub browsed_font_id: Option<Id>,
    pub sitemap_font_id: Option<Id>,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            sitemap_branch: Branch::DRAFT.into(),
            model_type: ModelType::Page,
            model_id: None,
            section: Section::Initial,
            file_id: None,
            browsed_font_id: None,
            sitemap_font_id: None,
        }
    }
}

pub struct ViewState {
    pub sitemap: Sitemap,
    pub model: Option<Model>,
    pub model_type: ModelType,
    pub section: Section,
    pub pages: Vec<Page>,
    pub layouts: Vec<Layout>,
    pub emails: Vec<Email>,
    pub files: Option<Vec<File>>,
    pub file: Option<File>,
    pub sitemap_fonts: Option<Vec<SitemapFont>>,
    pub sitemap_font: Option<SitemapFont>,
    pub browsed_fonts: Option<Vec<Font>>,
    pub browsed_font: Option<Font>,
    pub browsed_font_limit: Option<u16>,
    pub browsed_font_offset: Option<u16>,
    pub browsed_font_query: Option<String>,
    pub colors: Option<Vec<Color>>,
}

#[derive(Deserialize, Default)]
pub struct QueryState {
    pub section: Option<Section>,
    pub model_type: Option<ModelType>,
    pub model_id: Option<Id>,
    pub file_id: Option<Id>,
    pub browsed_fonts_query: Option<String>,
    pub browsed_fonts_limit: Option<u16>,
    pub browsed_fonts_offset: Option<u16>,
    pub browsed_font_id: Option<Id>,
    pub sitemap_font_id: Option<Id>,
}

#[derive(EnumIter, Debug, PartialEq, Serialize, Deserialize)]
pub enum Section {
    Initial,
    Create,
    Delete,
    Files,
    File,
    Fonts,
    BrowseFonts,
    ConfigureFont,
    Colors,
    EditStyles,
    EditScript,
    EditHTML,
    Publish,
}

impl Section {
    pub fn is_tab(&self) -> bool {
        match &self {
            Self::Initial => true,
            Self::Create => true,
            Self::Delete => true,
            Self::Files => true,
            Self::Fonts => true,
            Self::Colors => true,
            Self::EditHTML => true,
            Self::EditScript => true,
            Self::EditStyles => true,
            Self::Publish => true,
            _ => false,
        }
    }

    pub fn is_only_web(&self) -> bool {
        false
    }

    pub fn icon(&self) -> Option<&'static str> {
        match &self {
            Self::Initial => Some("fa-solid fa-house"),
            Self::Create => Some("fa-solid fa-plus"),
            Self::Delete => Some("fa-solid fa-trash-can"),
            Self::Files => Some("fa-solid fa-image"),
            Self::Fonts => Some("fa-solid fa-font"),
            Self::Colors => Some("fa-solid fa-palette"),
            Self::EditHTML => Some("fa-solid fa-code"),
            Self::EditStyles => Some("fa-brands fa-css"),
            Self::EditScript => Some("fa-brands fa-js"),
            Self::Publish => Some("fa-solid fa-upload"),
            _ => None,
        }
    }

    pub fn markup(&self, state: &ViewState) -> Markup {
        match &self {
            Section::Initial => initial(state),
            Section::Create => create(),
            Section::Delete => delete(),
            Section::Files => files(state),
            Section::File => file(state),
            Section::Fonts => fonts(&state.sitemap_fonts),
            Section::BrowseFonts => browse_fonts(state),
            Section::ConfigureFont => configure_font(state),
            Section::Colors => colors(state),
            _ => html!(),
        }
    }
}

pub fn content(state: &ViewState) -> Markup {
    html!(
        #content "data-scope"="pages" {
            (editor(&state))
            (preview(&state))
        }
    )
}

pub fn editor(state: &ViewState) -> Markup {
    let sections = Section::iter().filter(|s| -> bool {
        if !s.is_tab() {
            return false;
        }

        let state_is_web = match state.model {
            Some(Model::Page(_)) => true,
            Some(Model::Layout(_)) => true,
            _ => false,
        };

        if s.is_only_web() && !state_is_web {
            return false;
        }

        true
    });

    html!(
        article #editor .editor {
            div role="group" {
                @for section in sections {
                    (tab_button(&state, &section))
                }
            }
            div style="height: 100%; overflow: hidden;" {
                (state.section.markup(&state))
            }
        }
    )
}

pub fn preview(state: &ViewState) -> Markup {
    html!()
}

fn tab_button(state: &ViewState, section: &Section) -> Markup {
    let active = state.section == *section;
    let redirect = !active;
    html!(
        button.active[active]
                hx-get=[redirect.then_some("/dashboard/pages")]
                hx-vals=[redirect.then_some(json!({ "section": section }))]
                hx-target=[redirect.then_some("#editor")]
                hx-swap=[redirect.then_some("outerHTML")]
        {
            @if let Some(icon) = section.icon() {
                i.(icon) {}
            }
        }
    )
}

pub fn initial(state: &ViewState) -> Markup {
    let selected_is_page = state.model_type == ModelType::Page;
    let selected_is_layout = state.model_type == ModelType::Layout;
    let selected_is_email = state.model_type == ModelType::Email;

    html!(
        fieldset role="group" {
            legend { "Tipo de Plantilla" }
            select
                name="model_type"
                autocomplete="off"
                hx-get="/dashboard/pages"
                hx-target="#content"
                hx-swap="outerHTML" {
                option value="page" selected=[selected_is_page.then_some("")] { "Sitio" }
                option value="layout" selected=[selected_is_layout.then_some("")] { "Diseño" }
                option value="email" selected=[selected_is_email.then_some("")] { "Correo" }
            }
        }
        fieldset {
            legend { "Seleccionar Plantilla" }
            select name="model_id" hx-get="/dashboard/pages" hx-target="#content" hx-swap="outerHTML" required {
                @match state.model_type {
                    ModelType::Page => {
                        @for page in &state.pages {
                            option value=(&page.id) { (page.name) }
                        }
                    }
                    ModelType::Layout => {
                        @for layout in &state.layouts {
                            option value=(&layout.id) { (layout.name) }
                        }
                    }
                    ModelType::Email => {
                        @for email in &state.emails {
                            option value=(&email.id) { (email.name) }
                        }
                    }
                }
            }
        }
        @match &state.model {
            Some(Model::Page(page)) => {
                form hx-patch="/dashboard/pages/basic" hx-swap="none" {
                    fieldset {
                        legend { "Nombre" }
                        input name="name" required value=(page.name) {}
                    }
                    fieldset role="group" {
                        legend { "Url" }
                        input name="path" required value=(page.path) {}
                    }
                    fieldset {
                        legend { "Titulo" }
                        input name="title" required value=(page.title) {}
                    }
                    fieldset {
                        legend { "Imagen" }
                        input name="og_image" value=(page.og_image) {}
                    }
                    fieldset {
                        legend { "Tipo" }
                        select name="og_type" {
                            option value="" {"Ninguno"}
                            option value="website" { "Sitio Web" }
                            option value="article" { "Artículo" }
                            option value="profile" { "Perfil" }
                            option value="product" { "Producto" }
                        }
                    }
                    fieldset {
                        legend { "Descripción" }
                        textarea name="og_description" class="no-resize" rows="2" autocomplete="off" cols="10" {
                            (page.og_description)
                        }
                    }
                    fieldset role="group" {
                        legend { "Diseño" }
                        select name="layout" autocomplete="off" {
                            option value="" { "Ninguno" }
                            @for layout in &state.layouts {
                                option value=(layout.id) { (layout.name) }
                            }
                        }
                    }
                    .actions {
                        button type="submit" { "Guardar" }
                    }
                }
            },
            Some(Model::Layout(layout)) => {
                form hx-patch="/dashboard/pages/basic" hx-swap="none" {
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
                form hx-patch="/dashboard/pages/basic" hx-swap="none" {
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

pub fn create() -> Markup {
    html!(
        form hx-post="/dashboard/pages" hx-target="#content" hx-swap="outerHTML" {
            fieldset role="group" {
                   legend {"Tipo de Plantilla"}
                select name="type" {
                       option value="Page" { "Sitio" }
                    option value="Layout" {"Diseño"}
                }
            }

            div {
                fieldset {
                       legend { "Nombre" }
                    input name="name" required {}
                }
                fieldset {
                    legend { "Titulo" }
                    input name="title" required {}
                }
            }

            .actions {
                button {"Crear"}
            }
        }
    )
}

pub fn delete() -> Markup {
    html!(
        .delete {
            p {  "¿Estás seguro de que deseas eliminar este sitio?" }
            .actions {
                button hx-get="/dashboard/pages" hx-target="#editor" hx-swap="outerHTML" { "Cancelar" }
                button.danger hx-delete="/dashoboard/pages" hx-target="#content" hx-swap="outerHTML" {
                    "Eliminar"
                }
            }
        }
    )
}

pub fn files(state: &ViewState) -> Markup {
    html!(
        .grow.files {
            .actions x-data="progress"{
                form
                    hx-trigger="change from:input changed"
                    hx-post="/dashboard/pages/files"
                    hx-target="#files"
                    hx-encoding="multipart/form-data"
                    "@htmx:xhr:progress"="progress($event)"
                {
                    input type="file" x-ref="input" hidden name="files" accept="image/*,video/*" multiple {}
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
                @if let Some(files) = &state.files {
                    (file_grid(files))
                }
            }
        }
    )
}

pub fn file_grid(files: &Vec<File>) -> Markup {
    html!(
        @for file in files {
            .item id=(file.id)  {
                div x-data {
                    .hover
                        title=(file.name)
                        hx-get="/dashboard/pages"
                        hx-target="#editor"
                        hx-swap="outerHTML"
                        hx-vals=(json!({ "file_id": file.id, "section": Section::File }))
                        "@mouseenter"="$refs.video?.play()"
                        "@mouseleave"="$refs.video?.pause()"  {}

                    @if file.preset == "image" {
                        img src=(format!("/assets/dynamic/files/{}", file.name)) title=(file.name) alt=(file.name) {}
                    }
                    @if file.preset == "video" {
                        video x-ref="video" src=(format!("/assets/dynamic/files/{}", file.name)) title=(file.name) alt=(file.name) muted {}
                    }
                }
            }
        }
    )
}

pub fn file(state: &ViewState) -> Markup {
    html! (
        .grow.files {
            @if let Some(file) = &state.file {
                (file_detail(file))
            }
        }
    )
}

pub fn file_detail(file: &File) -> Markup {
    html!(
        .edit {
            .actions {
                button
                    type="button"
                    hx-trigger="click, deleted from:body, renamed from:body"
                    hx-get="/dashboard/pages"
                    hx-vals=(json!({ "section": Section::Files }))
                    hx-target="#editor"
                    hx-swap="outerHTML"
                {
                    "Volver"
                }
            }
            .preview {
                @if file.preset == "image" {
                        img src=(format!("/assets/dynamic/files/{}", file.name)) title=(file.name) alt=(file.name) {}
                }
                @if file.preset == "video" {
                    video src=(format!("/assets/dynamic/files/{}", file.name)) title=(file.name) alt=(file.name) controls {}
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
            form hx-patch=(format!("/dashboard/files/{}", file.name)) hx-swap="none" {
                fieldset {
                    legend { "Nombre" }
                    input name="name" value=(file.name) required {}
                }
                .actions.around {
                    button.danger type="button" hx-delete=(format!("/dashboard/files/{}", file.name)) hx-swap="none" class="danger" { "Eliminar" }
                    button type="submit" { "Guardar" }
                }
            }
        }
    )
}

pub fn fonts(sitemap_fonts: &Option<Vec<SitemapFont>>) -> Markup {
    html! (
        @if let Some(sitemap_fonts) = sitemap_fonts {
            .fonts {
                .list {
                    @for sitemap_font in sitemap_fonts {
                        .font
                            title=(sitemap_font.font_family)
                            hx-get="/dashboard/pages"
                            hx-target="#editor"
                            hx-swap="outerHTML"
                            hx-vals=(json!({ "section": Section::BrowseFonts, "sitemap_font_id": sitemap_font.id }))
                        {
                            .preview
                                x-data=(
                                    format!("font({{ family: {:?}, url: {:?} }})",
                                    sitemap_font.font_family, sitemap_font.font_files["regular"]
                                ))
                                ":style"="style"
                            {
                                (sitemap_font.font_family)
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
                        hx-get="/dashboard/pages"
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
                    hx-get="/dashboard/pages"
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
                    hx-get="/dashboard/pages"
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
                    hx-get=[is_last.then_some("/dashboard/pages")]
                    hx-trigger=[is_last.then_some("intersect once")]
                    hx-swap=[is_last.then_some("beforeend")]
                    hx-indicator=[is_last.then_some(".fonts")]
                    hx-target=[is_last.then_some("#browsed-fonts")]
                    hx-vals=[is_last.then_some(json!({ "browsed_fonts_query": query.clone().unwrap_or("".into()), "browsed_fonts_limit": limit.unwrap_or(10), "browsed_fonts_offset": offset.unwrap_or(0) + limit.unwrap_or(10) }))]
                {
                    div
                        hx-get="/dashboard/pages"
                        hx-target="#editor"
                        hx-swap="outerHTML"
                        hx-vals:append=(json!({ "browsed_font_id": font.id }))
                    {
                        .preview
                            x-data=(format!(
                                "font({{ family: {:?}, url: {:?} }})",
                                font.family, font.files["regular"]
                            ))
                            ":style"="style"
                        {
                            (font.family)
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
                            hx-get="/dashboard/pages"
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
                    form hx-post="/dashboard/pages" hx-target=".fonts" hx-swap="outerHTML" {
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

                        input name="action" value="save_font" hidden {}

                        div class="actions" {
                            button type="submit" { "Guardar" }
                        }
                    }
                }
            }
        }
    )
}

pub fn colors(state: &ViewState) -> Markup {
    html! (

        @let colors = match &state.colors {
          Some(colors) => &colors,
          None => &Vec::new()
        };

        @let swatches: Vec<&String> = colors.iter().map(|c| &c.value).collect();
        .colors x-data=(json!({"swatches": swatches})) {
            #colors {
                @for c in colors.iter() {
                    (color(c))
                }
            }
            .actions {
                button class="clear" hx-post="/dashboard/pages" hx-include="find input" hx-target="#colors" hx-swap="beforeend" {
                    input name="action" value="create_color" hidden {}
                    i class="fa-solid fa-plus" {}
                }
            }
        }

    )
}

pub fn color(color: &Color) -> Markup {
    html! {
        .color
            x-data=(format!(
                "coloris({{ color: {:?}, tag: {:?}, swatches }})",
                color.value, color.tag
            ))
        {
            form
                hx-post="/dashboard/pages"
                hx-trigger="input throttle:100ms"
                hx-swap="none"
            {
                .field x-bind:style="{color: readable, background: color}" {
                    input name="action" value="update_color" hidden {}
                    input name="id" value=(&color.id) hidden {}
                    input name="tag" x-model="tag" required {}
                    input
                        name="value"
                        class="coloris"
                        x-ref="input"
                        x-model="color"
                        x-bind:style="{ color: readable }"
                        required
                        {}
                }
            }
            small.hint {
                .css {
                    i.fa-brands.fa-css {}
                    pre {
                        "var(--color--" span x-text="tag" { (color.tag) } ")"
                    }
                }
                button.clear.danger
                    type="button"
                    hx-post="/dashboard/pages"
                    hx-target="closest .color"
                    hx-swap="outerHTML"
                    hx-vals=(json!({ "action": "delete_color", "id": &color.id }))
                {
                    i class="fa-solid fa-trash" {}
                }
            }
        }
    }
}
