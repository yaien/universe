use maud::{Markup, PreEscaped, html};

use crate::app::{Color, SitemapFont};

pub const BASE: &str = r#"
*,
*:before,
*:after {
    box-sizing: border-box;
}

body {
    margin: 0;
    padding: 0;
    background-color: var(--color-background);
    color: var(--color-text);
    font-family: var(--font-primary);
}

h1,
h2,
h3,
h4,
h5,
h6 {
    font-family: var(--font-headings);
}
"#;

pub fn root_vars(fonts: &Vec<SitemapFont>, colors: &Vec<Color>) -> String {
    let mut prescaped = String::from("\n:root {\n");

    for font in fonts {
        prescaped.push_str(&format!("  --font-{}: {:?};\n", font.tag, font.family));
    }

    for color in colors {
        prescaped.push_str(&format!("  --color-{}: {};\n", color.tag, color.value));
    }

    prescaped.push_str("}\n");

    prescaped
}

pub fn inline(
    fonts: &Vec<SitemapFont>,
    colors: &Vec<Color>,
    layout_styles: &str,
    page_styles: &str,
) -> Markup {
    let vars = root_vars(fonts, colors);

    let prescaped = format!(
        r#"
        <style type="text/css">
            {vars}
            {BASE}
            {page_styles}
            {layout_styles}
        </style>
        "#,
    );

    html! {
        (PreEscaped(prescaped))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn sitemap_font(tag: &str, family: &str) -> SitemapFont {
        SitemapFont {
            id: 0,
            family: family.to_string(),
            tag: tag.to_string(),
            font_id: 0,
            provider: "".to_string(),
            variants: vec![],
            files: HashMap::new(),
        }
    }

    fn color(tag: &str, value: &str) -> Color {
        Color {
            id: 0,
            sitemap_id: 0,
            tag: tag.to_string(),
            value: value.to_string(),
        }
    }

    #[test]
    fn test_root_vars() {
        let fonts = vec![
            sitemap_font("primary", "Arial"),
            sitemap_font("secondary", "Helvetica"),
        ];
        let colors = vec![color("primary", "#000000"), color("secondary", "#FFFFFF")];
        let result = root_vars(&fonts, &colors);
        let expected = r##"
            :root {
               --font-primary: "Arial";
               --font-secondary: "Helvetica";
               --color-primary: #000000;
               --color-secondary: #FFFFFF;
            }
        "##;

        for (index, value) in result.lines().enumerate() {
            assert_eq!(value.trim(), expected.lines().nth(index).unwrap().trim());
        }
    }
}
