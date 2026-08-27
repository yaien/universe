use maud::{Markup, html};
use serde::{Deserialize, Serialize};
use serde_json::json;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::app::{
    Branch, Color, Email, File, Font, Layout, Organization, Page, Sitemap, SitemapFont,
};
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

pub struct ViewState<'a> {
    pub organization: &'a Organization,
    pub sitemap: Sitemap,
    pub sitemaps: Option<Vec<Sitemap>>,
    pub model: Option<Model>,
    pub model_type: ModelType,
    pub section: Section,
    pub pages: Option<Vec<Page>>,
    pub layouts: Option<Vec<Layout>>,
    pub emails: Option<Vec<Email>>,
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
    pub sitemap_branch: Option<String>,
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
    Edit,
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
            Self::Edit => true,
            Self::Files => true,
            Self::Fonts => true,
            Self::Colors => true,
            Self::EditHTML => true,
            Self::EditScript => true,
            Self::EditStyles => true,
            _ => false,
        }
    }

    pub fn is_only_web(&self) -> bool {
        match &self {
            Self::Fonts => true,
            Self::Colors => true,
            Self::EditScript => true,
            Self::EditStyles => true,
            _ => false,
        }
    }

    pub fn icon(&self) -> Option<&'static str> {
        match &self {
            Self::Initial => Some("fa-solid fa-house"),
            Self::Edit => Some("fa-solid fa-pen"),
            Self::Files => Some("fa-solid fa-image"),
            Self::Fonts => Some("fa-solid fa-font"),
            Self::Colors => Some("fa-solid fa-palette"),
            Self::EditHTML => Some("fa-solid fa-code"),
            Self::EditStyles => Some("fa-brands fa-css"),
            Self::EditScript => Some("fa-brands fa-js"),
            _ => None,
        }
    }

    pub fn markup(&self, state: &ViewState) -> Markup {
        match &self {
            Section::Initial => initial(state),
            Section::Edit => edit(&state.model, &state.organization, &state.layouts),
            Section::Create => create(state),
            Section::Delete => delete(state),
            Section::Publish => publish(),
            Section::Files => files(&state.files),
            Section::File => file(state),
            Section::Fonts => fonts(&state.sitemap_fonts),
            Section::BrowseFonts => browse_fonts(state),
            Section::ConfigureFont => configure_font(state),
            Section::Colors => colors(state),
            Section::EditHTML => edit_html(state),
            Section::EditScript => edit_js(state),
            Section::EditStyles => edit_css(state),
        }
    }
}

pub fn content(state: &ViewState) -> Markup {
    html!(
        #content "data-scope"="pages" {
            (editor(&state))
            (preview(false))
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
            #section.section {
                (state.section.markup(&state))
            }
        }
    )
}

pub fn preview(swap: bool) -> Markup {
    html!(
        #preview.page hx-swap-oob=[swap.then_some("true")] {
            .resizeable {
                iframe x-ref="iframe" src="/dashboard/pages/preview" {}
            }
        }
    )
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
    let selected_is_page = (state.model_type == ModelType::Page).then_some("");
    let selected_is_layout = (state.model_type == ModelType::Layout).then_some("");
    let selected_is_email = (state.model_type == ModelType::Email).then_some("");

    html!(
        form autocomplete="off" {
            @if let Some(sitemaps) = &state.sitemaps {
                fieldset role="group" {
                    legend { "Mapa de Sitio"}
                    select
                        name="sitemap_branch"
                        autocomplete="off"
                        hx-get="/dashboard/pages"
                        hx-target="#content"
                        hx-swap="outerHTML" {

                        @for sitemap in sitemaps {
                            option value=(sitemap.branch) selected=[(sitemap.id == state.sitemap.id).then_some("")] { (sitemap.branch) }
                        }
                    }

                }
            }
            fieldset role="group" {
                legend { "Tipo de Plantilla" }
                select
                    name="model_type"
                    autocomplete="off"
                    hx-get="/dashboard/pages"
                    hx-target="#content"
                    hx-swap="outerHTML" {
                    option value="page" selected=[selected_is_page] { "Sitio" }
                    option value="layout" selected=[selected_is_layout] { "Diseño" }
                    option value="email" selected=[selected_is_email] { "Correo" }
                }
            }
            fieldset {
                legend { "Seleccionar Plantilla" }
                select name="model_id" hx-get="/dashboard/pages" hx-target="#content" hx-swap="outerHTML" required  {
                    @match &state.model {
                        Some(Model::Page(selected)) => {
                            @if let Some(pages) = &state.pages {
                                @for page in pages {
                                    option value=(&page.id) selected=[(page.id == selected.id).then_some("true")] { (page.name) }
                                }
                            }
                        }
                        Some(Model::Layout(selected)) => {
                            @if let Some(layouts) = &state.layouts {
                                @for layout in layouts {
                                    option value=(&layout.id) selected=[(layout.id == selected.id).then_some("")] { (layout.name) }
                                }
                            }
                        }
                        Some(Model::Email(selected)) => {
                            @if let Some(emails) = &state.emails {
                                @for email in emails {
                                    option value=(&email.id) selected=[(email.id == selected.id).then_some("")] { (email.name) }
                                }
                            }
                        }
                        _=> {}

                    }
                }
            }
        }



        div role="group" {
            button title="Crear" hx-get="/dashboard/pages" hx-target="#editor" hx-swap="outerHTML"  hx-vals=(json!({"section": Section::Create}))  {
                i.fa-solid.fa-plus {}
            }

            button title="Eliminar" hx-get="/dashboard/pages" hx-target="#editor" hx-swap="outerHTML"  hx-vals=(json!({"section": Section::Delete}))  {
                i.fa-solid.fa-trash {}
            }

            button title="Publicar" hx-get="/dashboard/pages" hx-target="#editor" hx-swap="outerHTML"  hx-vals=(json!({"section": Section::Publish}))  {
                i.fa-solid.fa-upload {}
            }
        }

    )
}

pub fn create(state: &ViewState) -> Markup {
    html!(
        .actions {
            button
                type="button"
                hx-trigger="click, deleted from:body, renamed from:body"
                hx-get="/dashboard/pages"
                hx-vals=(json!({ "section": Section::Initial}))
                hx-target="#editor"
                hx-swap="outerHTML"
            {
                "Volver"
            }
        }
        fieldset x-data="{ modelType: 'page' }" {
            legend { "Agregar Modelo" }
            form hx-post="/dashboard/pages" hx-target="#section" {
                fieldset role="group" {
                    legend {"Tipo de Plantilla"}
                    select name="model_type" x-model="modelType" {
                        option value="page" { "Sitio" }
                        option value="layout" {"Diseño"}
                    }
                }

                template x-if="modelType === 'page'" {
                    div {
                        fieldset {
                            legend { "Path" }
                            .group {
                                span { (state.organization.url) }
                                input name="path" required {}
                            }
                        }
                        fieldset {
                            legend { "Nombre" }
                            input name="name" required {}
                        }
                        fieldset {
                            legend { "Titulo" }
                            input name="title" required {}
                        }
                        input name="action" value="create_page" hidden {}
                    }
                }

                template x-if="modelType === 'layout'" {
                    div {
                        fieldset {
                            legend { "Nombre" }
                            input name="name" required {}
                        }
                        input name="action" value="create_layout" hidden {}
                    }
                }

                .actions {
                    button {"Crear"}
                }
            }
        }
        fieldset {
            legend { "Sincronizar Mapa de Sitio" }
            form hx-post="/dashboard/pages" hx-target="#content" hx-swap="outerHTML" {
                fieldset {
                    legend { "Nombre" }
                    .group {
                        span { "draft/"}
                        input name="name" required {}
                    }
                }


                input name="action" value="sync_draft" hidden {}

                .actions {
                    button {"Sincronizar"}
                }
            }
        }
    )
}

pub fn delete(state: &ViewState) -> Markup {
    html!(
        .actions {
            button
                type="button"
                hx-trigger="click, deleted from:body, renamed from:body"
                hx-get="/dashboard/pages"
                hx-vals=(json!({ "section": Section::Initial}))
                hx-target="#editor"
                hx-swap="outerHTML"
            {
                "Volver"
            }
        }
        .delete {
            @let count = match &state.model_type {
                ModelType::Page => state.pages.as_ref().map(|p| p.len()).unwrap_or(0 as usize),
                ModelType::Layout => state.layouts.as_ref().map(|l| l.len()).unwrap_or(0 as usize),
                _ => 0,
            };

            @if count > 1 {
                fieldset {
                    legend { "Eliminar plantilla" }
                    p {
                        @match &state.model {
                            Some(Model::Page(page)) => {
                                "Estás seguro de que deseas eliminar la página " b { (page.name) } "?"
                            },
                            Some(Model::Layout(layout)) => {
                                "Estás seguro de que deseas eliminar el layout " b { (layout.name) } "?"
                            },
                            Some(Model::Email(email)) => {
                                "Estás seguro de que deseas eliminar el email " b { (email.name) } "?"
                            },
                            _ => ""
                        }
                    }
                    .actions {
                        @let action = match &state.model_type {
                            ModelType::Page => "delete_page",
                            ModelType::Layout => "delete_layout",
                        _ => "",
                        };
                        button.danger hx-post="/dashboard/pages" hx-target="#content" hx-swap="outerHTML" hx-vals=(json!({ "action": action})) {
                            "Eliminar"
                        }
                    }
                }
            }
            @if let Some(sitemaps) = &state.sitemaps {
                @if sitemaps.len() > 1 && state.sitemap.branch != Branch::DRAFT {
                    fieldset {
                        legend { "Eliminar mapa de sitio" }
                        p {
                            "Estás seguro de que deseas eliminar el mapa de sitio " b { (state.sitemap.branch) } "?"
                        }
                        .actions {
                            button.danger hx-post="/dashboard/pages" hx-target="#content" hx-swap="outerHTML" hx-vals=(json!({ "action": "delete_sitemap"})) {
                                "Eliminar"
                            }
                        }
                    }
                }
            }
        }
    )
}

pub fn publish() -> Markup {
    html!(
        .actions {
            button
                type="button"
                hx-trigger="click, deleted from:body, renamed from:body"
                hx-get="/dashboard/pages"
                hx-vals=(json!({ "section": Section::Initial}))
                hx-target="#editor"
                hx-swap="outerHTML"
            {
                "Volver"
            }
        }
        .publish {
            p {
                "Al publicar el mapa de sitio actual, este estará disponible para los usuarios finales"
            }
            .actions {
                button.warning hx-post="/dashboard/pages" hx-swap="none" hx-vals=(json!({ "action": "publish"})) { "Publicar" }
            }
        }
    )
}

pub fn edit(model: &Option<Model>, org: &Organization, layouts: &Option<Vec<Layout>>) -> Markup {
    html!(
        @match model {
            Some(Model::Page(page)) => {
                form hx-post="/dashboard/pages" hx-swap="none" autocomplete="off" {
                    fieldset {
                        legend { "Nombre" }
                        input name="name" required value=(page.name) {}
                    }
                    fieldset role="group" {
                        legend { "Url" }
                        .group {
                            span { (org.url) }
                            input name="path" required value=(page.path) {}
                        }
                    }
                    fieldset {
                        legend { "Titulo" }
                        input name="title" required value=(page.title) {}
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

                    input name="action" type="hidden" value="save_page_info" {}

                    .actions {
                        button type="submit" { "Guardar" }
                    }
                }
            },
            Some(Model::Layout(layout)) => {
                form hx-post="/dashboard/pages" hx-swap="none" {
                    fieldset {
                        legend { "Nombre" }
                        input name="name" required value=(layout.name) {}
                    }
                    input name="action" type="hidden" value="save_layout_info" {}
                    .actions {
                        button type="submit" { "Guardar" }
                    }
                }
            },
            Some(Model::Email(email)) => {
                form hx-post="/dashboard/pages" hx-swap="none" {
                    fieldset {
                        legend { "Asunto" }
                        textarea name="subject" class="no-resize" required cols="10" {
                            (email.subject)
                        }
                    }
                    input name="action" type="hidden" value="save_email_info" {}
                    .actions {
                        button type="submit" { "Guardar" }
                    }
                }
            },
            None => {}
        }
    )
}

pub fn files(files: &Option<Vec<File>>) -> Markup {
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
                    input type="file" x-ref="input" hidden name="files" accept="image/png,image/jpeg,image/jpg,video/mp4" multiple {}
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
                @if let Some(files) = files {
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
        #edit.edit {
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
            form hx-post="/dashboard/pages" hx-swap="none" {
                input type="hidden" name="action" value="save_file" {}

                fieldset {
                    legend { "Nombre" }
                    input name="name" value=(file.name) required {}
                }
                .actions.around {
                    button.danger type="button" hx-post="/dashboard/pages" hx-target="#edit" hx-swap="outerHTML" hx-vals=(json!({ "action": "delete_file" })) {
                        "Eliminar"
                    }
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
                            title=(sitemap_font.family)
                            hx-get="/dashboard/pages"
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

pub fn edit_html(state: &ViewState) -> Markup {
    let source: &str = match &state.model {
        Some(Model::Page(page)) => &page.html,
        Some(Model::Layout(layout)) => &layout.html,
        Some(Model::Email(email)) => &email.body,
        _ => "",
    };

    html! {
        .monaco
            x-data=(format!(
                "monaco({{ language: {:?}, source: {:?} }})",
                "html", source
            ))
            hx-post="/dashboard/pages"
            hx-trigger="editorinput"
            hx-vals=(format!(
                "js:{{ action: {:?}, source: event.detail.value }}",
                "save_html"
            ))
            hx-swap="none"
        {
            .spinner x-show="loading" {
                i class="fa-solid fa-spinner" {}
            }
        }
    }
}

pub fn edit_css(state: &ViewState) -> Markup {
    let source: &str = match &state.model {
        Some(Model::Page(page)) => &page.css,
        Some(Model::Layout(layout)) => &layout.css,
        _ => "",
    };

    html! {
        .monaco
            x-data=(format!(
                "monaco({{ language: {:?}, source: {:?} }})",
                "css", source
            ))
            hx-post="/dashboard/pages"
            hx-trigger="editorinput"
            hx-vals=(format!(
                "js:{{ action: {:?}, source: event.detail.value }}",
                "save_css"
            ))
            hx-swap="none"
        {
            .spinner x-show="loading" {
                i class="fa-solid fa-spinner" {}
            }
        }
    }
}

pub fn edit_js(state: &ViewState) -> Markup {
    let source: &str = match &state.model {
        Some(Model::Page(page)) => &page.js,
        Some(Model::Layout(layout)) => &layout.js,
        _ => "",
    };

    html! {
        .monaco
            x-data=(format!(
                "monaco({{ language: {:?}, source: {:?} }})",
                "javascript", source
            ))
            hx-post="/dashboard/pages"
            hx-trigger="editorinput"
            hx-vals=(format!(
                "js:{{ action: {:?}, source: event.detail.value }}",
                "save_js"
            ))
            hx-swap="none"
        {
            .spinner x-show="loading" {
                i class="fa-solid fa-spinner" {}
            }
        }
    }
}
