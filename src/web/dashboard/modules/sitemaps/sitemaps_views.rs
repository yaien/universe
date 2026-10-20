mod sitemaps_actions_views;
mod sitemaps_colors_views;
mod sitemaps_edit_views;
mod sitemaps_files_views;
mod sitemaps_fonts_views;
mod sitemaps_form_views;
mod sitemaps_sources_views;
mod sitemaps_vars_views;
mod sitempas_initial_views;

use maud::{Markup, html};
use serde::{Deserialize, Serialize};
use serde_json::json;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::app::auth::Organization;
use crate::app::forms::Form;
use crate::app::sitemaps::{
    Action, Branch, Color, Email, Font, Layout, Page, Sitemap, SitemapFont, Var,
};
use crate::app::storage::File;
use crate::infra::Id;

pub use sitemaps_actions_views::*;
pub use sitemaps_colors_views::*;
pub use sitemaps_edit_views::*;
pub use sitemaps_files_views::*;
pub use sitemaps_fonts_views::*;
pub use sitemaps_form_views::*;
pub use sitemaps_sources_views::*;
pub use sitemaps_vars_views::*;
pub use sitempas_initial_views::*;

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
    pub action_id: Option<Id>,
    pub form_id: Option<Id>,
    pub var_id: Option<Id>,
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
            action_id: None,
            form_id: None,
            var_id: None,
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
    pub actions: Option<Vec<Action>>,
    pub action: Option<Action>,
    pub forms: Option<Vec<Form>>,
    pub form: Option<Form>,
    pub vars: Option<Vec<Var>>,
    pub var: Option<Var>,
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
    pub action_id: Option<Id>,
    pub form_id: Option<Id>,
    pub var_id: Option<Id>,
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
    Actions,
    CreateAction,
    EditAction,
    Vars,
    CreateVar,
    EditVar,
    Forms,
    CreateForm,
    EditForm,
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
            Self::Actions => true,
            Self::Forms => true,
            Self::Vars => true,
            _ => false,
        }
    }

    pub fn is_only_web(&self) -> bool {
        match &self {
            Self::Fonts => true,
            Self::Colors => true,
            Self::EditScript => true,
            Self::EditStyles => true,
            Self::EditAction => true,
            Self::Vars => true,
            Self::Forms => true,
            Self::Actions => true,
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
            Self::Actions => Some("fa-solid fa-hand-point-up"),
            Self::Forms => Some("fa-solid fa-table-list"),
            Self::Vars => Some("fa-solid fa-atom"),

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
            Section::Files => files(&state),
            Section::File => file(&state),
            Section::Fonts => fonts(&state.sitemap_fonts),
            Section::BrowseFonts => browse_fonts(state),
            Section::ConfigureFont => configure_font(state),
            Section::Colors => colors(state),
            Section::EditHTML => edit_html(state),
            Section::EditScript => edit_js(state),
            Section::EditStyles => edit_css(state),
            Section::Actions => actions(state),
            Section::CreateAction => create_action(),
            Section::EditAction => edit_action(&state.action),
            Section::Forms => forms(&state.forms),
            Section::CreateForm => create_form(),
            Section::EditForm => edit_form(&state.form),
            Section::Vars => vars(state),
            Section::CreateVar => create_var(state),
            Section::EditVar => edit_var(&state.var),
        }
    }
}

pub fn content(state: &ViewState) -> Markup {
    html!(
        #content "data-scope"="pages" {
            (editor(&state))
            (preview())
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

pub const RELOAD_HEADER: (&str, &str) =
    ("HX-Trigger", r##"{ "reload": { "target": "#preview" } }"##);

pub fn preview() -> Markup {
    html!(
        .page hx-preserve="true"{
            .resizeable {
                iframe #preview x-data="preview" src="/dashboard/sitemaps/preview" {}
            }
        }
    )
}

fn tab_button(state: &ViewState, section: &Section) -> Markup {
    let active = state.section == *section;
    let redirect = !active;
    html!(
        button.active[active]
                hx-get=[redirect.then_some("/dashboard/sitemaps")]
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
