use derive_more::Display;
use log::info;
use maud::{Markup, html};
use serde::{Deserialize, Serialize};
use serde_json::json;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::app::{Branch, Email, Layout, Page, Sitemap};
use crate::infra::Id;

pub enum Model {
    Page(Page),
    Layout(Layout),
    Email(Email),
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
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
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            sitemap_branch: Branch::DRAFT.into(),
            model_type: ModelType::Page,
            model_id: None,
            section: Section::Initial,
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
    let mut hx_get = None;
    let mut hx_target = None;
    let mut hx_swap = None;
    let mut hx_vals = None;
    if !active {
        hx_get = Some("/dashboard/pages");
        hx_target = Some("#editor");
        hx_swap = Some("outerHTML");
        hx_vals = Some(json!({ "section": section}))
    }
    html!(
        button.active[active]
                hx-get=[hx_get]
                hx-vals=[hx_vals]
                hx-target=[hx_target]
                hx-swap=[hx_swap]
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
                option value="Page" selected=[selected_is_page.then_some("")] { "Sitio" }
                option value="Layout" selected=[selected_is_layout.then_some("")] { "Diseño" }
                option value="Email" selected=[selected_is_email.then_some("")] { "Correo" }
            }
        }
        fieldset {
            legend { "Seleccionar Plantilla" }
            select name="model_id" hx-get="/dashboard/pages" hx-target="#content" hx-swap="outerHTML" required {
                @match state.model_type {
                    ModelType::Page => {
                        @for page in &state.pages {
                            option value=(&page.id) { (page.path) }
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
