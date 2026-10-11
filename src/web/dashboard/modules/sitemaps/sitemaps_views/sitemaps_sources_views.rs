use super::*;

pub fn edit_html(state: &ViewState) -> Markup {
    let source: &str = match &state.model {
        Some(Model::Page(page)) => &page.html,
        Some(Model::Layout(layout)) => &layout.html,
        Some(Model::Email(email)) => &email.body,
        _ => "",
    };

    html! {
        .monaco
            x-data=(format!(r#"monaco({{ language: "html", source: {source:?} }})"#))
            hx-patch="/dashboard/sitemaps/html"
            hx-trigger="editorinput"
            hx-vals="js:{ source: event.detail.value }"
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
            x-data=(format!(r#"monaco({{ language: "css", source: {source:?} }})"#))
            hx-patch="/dashboard/sitemaps/css"
            hx-trigger="editorinput"
            hx-vals="js:{ source: event.detail.value }"
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
            x-data=(format!(r#"monaco({{ language: "javascript", source: {source:?} }})"#))
            hx-patch="/dashboard/sitemaps/js"
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
