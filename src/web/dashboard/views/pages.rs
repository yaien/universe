use derive_more::Display;
use log::info;
use maud::{Markup, html};
use serde::{Deserialize, Serialize};
use serde_json::json;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::app::{Branch, Email, File, Layout, Page, Sitemap};
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
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            sitemap_branch: Branch::DRAFT.into(),
            model_type: ModelType::Page,
            model_id: None,
            section: Section::Initial,
            file_id: None,
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
}

#[derive(Deserialize)]
pub struct QueryState {
    pub section: Option<Section>,
    pub model_type: Option<ModelType>,
    pub model_id: Option<Id>,
    pub file_id: Option<Id>,
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
    ConfigureFonts,
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

    pub fn is_delete(&self) -> bool {
        match &self {
            Self::Delete => true,
            _ => false,
        }
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
            Section::Files => files(),
            Section::File => file(),
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

        info!("Pass Section, {:?}", s);

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

pub fn files() -> Markup {
    html!(
        .grow.files {
            .actions x-data="progress"{
                form
                    hx-trigger="change changed"
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
            #files .grid hx-get="/dashboard/pages/files" hx-trigger="load" {
                .htmx-indicator.spinner {
                    i.fa-solid.fa-spinner {}
                }
            }
        }
    )
}

pub fn file_grid(files: Vec<File>) -> Markup {
    html!(
        @for file in files {
            .item id=(file.id) hx-vals=(json!({ "file_id": file.id, "section": Section::File })) {
                div x-data {

                    .hover
                        title=(file.name)
                        hx-get="/dashboard/pages"
                        hx-target="#editor"
                        hx-swap="outerHTML"
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

pub fn file() -> Markup {
    html! (
        .grow.files hx-trigger="load" hx-get="/dashboard/pages/file" {
            .htmx-indicator.spinner {
                i.fa-solid.fa-spinner {}
            }
        }
    )
}

pub fn file_detail(file: &File) -> Markup {
    html!(
        .grow.files {
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
        }
    )
}
