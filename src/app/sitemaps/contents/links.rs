use maud::{Markup, html};

use crate::app::sitemaps::SitemapFont;

pub fn fonts(sitemap_fonts: &Vec<SitemapFont>) -> Markup {
    let has_google_fonts = sitemap_fonts.iter().any(|f| f.provider == "google");
    html!(
        @if has_google_fonts {
             link rel="preconnect" href="https://fonts.googleapis.com" {}
             link rel="preconnect" href="https://fonts.gstatic.com" crossorigin {}
        }
        @for font in sitemap_fonts {
            link rel="stylesheet" href=(google_font_url(&font.family, &font.variants)) {}
        }
    )
}

/// Parsea una variante tipo "regular", "italic", "500", "700italic"
/// y devuelve (peso, es_italica)
fn parse_google_variant(variant: &str) -> (u32, bool) {
    let is_italic = variant.ends_with("italic");

    let weight_part = variant.trim_end_matches("italic");

    let weight = if weight_part.is_empty() || weight_part == "regular" {
        400
    } else {
        weight_part.parse::<u32>().unwrap_or(400)
    };

    (weight, is_italic)
}

/// Construye la URL de Google Fonts (css2 API) a partir de un SitemapFont
pub fn google_font_url(family: &str, variants: &Vec<String>) -> String {
    let family_encoded = family.replace(' ', "+");

    let mut normal_weights: Vec<u32> = Vec::new();
    let mut italic_weights: Vec<u32> = Vec::new();

    for variant in variants {
        let (weight, is_italic) = parse_google_variant(variant);
        if is_italic {
            italic_weights.push(weight);
        } else {
            normal_weights.push(weight);
        }
    }

    let mut axis_parts: Vec<String> = Vec::new();

    if !normal_weights.is_empty() {
        let min = normal_weights.iter().min().unwrap();
        let max = normal_weights.iter().max().unwrap();
        if min == max {
            axis_parts.push(format!("0,{}", min));
        } else {
            axis_parts.push(format!("0,{}..{}", min, max));
        }
    }

    if !italic_weights.is_empty() {
        let min = italic_weights.iter().min().unwrap();
        let max = italic_weights.iter().max().unwrap();
        if min == max {
            axis_parts.push(format!("1,{}", min));
        } else {
            axis_parts.push(format!("1,{}..{}", min, max));
        }
    }

    let mut family_param = family_encoded;
    if !axis_parts.is_empty() {
        family_param.push_str(":ital,wght@");
        family_param.push_str(&axis_parts.join(";"));
    }

    format!(
        "https://fonts.googleapis.com/css2?family={}&display=swap",
        family_param
    )
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_google_fonts_url() {
        struct Test {
            name: &'static str,
            variants: &'static [&'static str],
            expected: &'static str,
        }

        let tests = [Test {
            name: "JetBrains Mono",
            variants: &[
                "100",
                "200",
                "300",
                "regular",
                "500",
                "600",
                "700",
                "800",
                "100italic",
                "200italic",
                "300italic",
                "italic",
                "500italic",
                "600italic",
                "700italic",
                "800italic",
            ],
            expected: "https://fonts.googleapis.com/css2?family=JetBrains+Mono:ital,wght@0,100..800;1,100..800&display=swap",
        }];

        for test in tests {
            let variants = test
                .variants
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>();

            let family = test.name;

            let url = google_font_url(&family, &variants);

            assert_eq!(url, test.expected, "failed for test {}", test.name);
        }
    }
}
