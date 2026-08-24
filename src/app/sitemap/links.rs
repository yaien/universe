use maud::{Markup, html};

use crate::app::SitemapFont;

pub fn fonts(sitemap_fonts: &Vec<SitemapFont>) -> Markup {
    let has_google_fonts = sitemap_fonts.iter().any(|f| f.provider == "google");
    html!(
        @if has_google_fonts {
             link rel="preconnect" href="https://fonts.googleapis.com" {}
             link rel="preconnect" href="https://fonts.gstatic.com" crossorigin {}
        }
        @for font in sitemap_fonts {
            link rel="stylesheet" href=(google_font_url(font)) {}
        }
    )
}

fn google_font_url(font: &SitemapFont) -> String {
    let mut url = String::from("https://fonts.googleapis.com/css2?");
    let family = font.family.replace(" ", "+");
    url.push_str("family=");
    url.push_str(&family);

    // Manejar variantes correctamente
    if !font.variants.is_empty() {
        let mut weights = Vec::new();
        let mut italic_weights = Vec::new();
        let mut has_italic = false;

        for variant in &font.variants {
            if variant == "italic" {
                has_italic = true;
                italic_weights.push("400");
                continue;
            }

            if variant.ends_with("italic") {
                has_italic = true;
                let weight = variant.trim_end_matches("italic");
                if weight.is_empty() {
                    italic_weights.push("400");
                } else {
                    italic_weights.push(weight);
                }
                continue;
            }

            if variant == "regular" {
                weights.push("400");
                continue;
            }

            weights.push(variant);
        }

        // Si no hay pesos específicos, agregar 400 por defecto
        if weights.is_empty() && !has_italic {
            weights.push("");
        }

        // Construir URL según las variantes
        if has_italic && !weights.is_empty() {
            // Formato: ital,wght@0,400;0,700;1,400;1,700
            url.push_str(":ital,wght@");
            let mut params = Vec::new();

            // Pesos normales
            for w in &weights {
                params.push("0,");
                params.push(w);
            }

            // Pesos itálicos (usar weights si italicWeights está vacío)
            let mut target_italic_weights = italic_weights;
            if target_italic_weights.is_empty() && !weights.is_empty() {
                target_italic_weights = weights
            }

            for w in target_italic_weights {
                params.push("1,");
                params.push(w);
            }

            url.push_str(&params.join(";"));
        } else if has_italic {
            // Solo itálica sin pesos específicos
            url.push_str(":ital,wght@1,400");
        } else if !weights.is_empty() {
            // Solo pesos sin itálica
            url.push_str(":wght@");
            url.push_str(&weights.join(";"));
        }
    } else {
        // Sin variantes especificadas - cargar básicas
        url.push_str(":wght@300;400;500;600;700");
    }

    // Agregar display=swap para mejor rendimiento
    url.push_str("&display=swap");

    url
}
