mod css;
mod js;

use lux_ast::analysis::AnalysisTables;
use lux_ast::template::root::Root;
use lux_utils::hash::hash;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformResult {
    pub js: String,
    pub css: Option<String>,
    pub css_hash: Option<String>,
    pub css_scope: Option<String>,
}

pub fn transform(root: &Root<'_>, analysis: &AnalysisTables) -> TransformResult {
    let (css, css_hash, css_scope) = match &root.css {
        Some(stylesheet) => {
            let css_hash = hash(stylesheet.content_styles);
            let css_scope = format!("svelte-{css_hash}");
            let css = css::render_stylesheet(stylesheet, analysis, &css_scope);
            (Some(css), Some(css_hash), Some(css_scope))
        }
        None => (None, None, None),
    };

    let js = js::render_component(
        root,
        css.as_deref(),
        css_hash.as_deref(),
        css_scope.as_deref(),
    );

    TransformResult {
        js,
        css,
        css_hash,
        css_scope,
    }
}
