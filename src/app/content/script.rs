use maud::{Markup, PreEscaped, html};

use crate::app::{Layout, Page};

pub fn inline(layout_script: &str, page_script: &str) -> Markup {
    let script = format!(
        r#"
        <script type="text/javascript">
            function __init() {{
                {layout_script}
                {page_script}
            }}
            document.addEventListener('alpine:init', __init);
        </script>

    "#,
    );
    html!((PreEscaped(script)))
}

pub fn bundle(layouts: &Vec<Layout>, pages: &Vec<Page>) -> String {
    let mut script = String::from("function __init() {\n");

    for layout in layouts {
        script.push_str(&format!("  {}\n", layout.js));
    }
    for page in pages {
        script.push_str(&format!("  {}\n", page.js));
    }
    script.push_str("}\n");
    script.push_str("document.addEventListener('alpine:init', __init);\n");
    script.push_str("if (window.Alpine) { __init(); }\n");

    script
}
