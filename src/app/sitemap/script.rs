use maud::{Markup, PreEscaped, html};

pub fn inline(layout_script: &str, page_script: &str) -> Markup {
    let script = format!(
        r#"
        <script type="text/javascript">
            function __init() {{
                {layout_script}
                {page_script}
            }}
            __init();
        </script>

    "#,
    );
    html!((PreEscaped(script)))
}
