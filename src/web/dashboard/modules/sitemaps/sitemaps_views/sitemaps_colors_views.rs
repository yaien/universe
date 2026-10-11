use super::*;

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
                button class="clear" hx-post="/dashboard/sitemaps/colors" hx-target="#colors" hx-swap="beforeend" {
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
                hx-put=(format!("/dashboard/sitemaps/colors/{}", color.id))
                hx-trigger="input throttle:100ms"
                hx-swap="none"
            {
                .field x-bind:style="{color: readable, background: color}" {
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
                    pre x-text="`var(--${tag}--color)`" {}
                }
                button.clear.danger
                    type="button"
                    hx-delete=(format!("/dashboard/sitemaps/colors/{}", color.id))
                    hx-target="closest .color"
                    hx-swap="outerHTML" {
                    i class="fa-solid fa-trash" {}
                }
            }
        }
    }
}
