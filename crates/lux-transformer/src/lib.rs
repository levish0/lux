mod css;
mod js;
mod runtime;

use lux_ast::analysis::AnalysisTables;
use lux_ast::template::root::Root;
use lux_utils::hash::hash;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeModule {
    pub specifier: String,
    pub code: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformResult {
    pub js: String,
    pub css: Option<String>,
    pub css_hash: Option<String>,
    pub css_scope: Option<String>,
    pub runtime_modules: Vec<RuntimeModule>,
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

    let component = js::render_component(
        root,
        analysis,
        css.as_deref(),
        css_hash.as_deref(),
        css_scope.as_deref(),
    );
    let runtime_modules = if component.needs_server_runtime {
        vec![RuntimeModule {
            specifier: runtime::SERVER_RUNTIME_SPECIFIER.to_string(),
            code: runtime::server_runtime_source().to_string(),
        }]
    } else {
        Vec::new()
    };

    TransformResult {
        js: component.js,
        css,
        css_hash,
        css_scope,
        runtime_modules,
    }
}
