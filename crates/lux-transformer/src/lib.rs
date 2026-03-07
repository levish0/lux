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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformTarget {
    Server,
    Client,
}

pub fn transform(root: &Root<'_>, analysis: &AnalysisTables) -> TransformResult {
    transform_with_filename(root, analysis, None)
}

pub fn transform_with_filename(
    root: &Root<'_>,
    analysis: &AnalysisTables,
    filename: Option<&str>,
) -> TransformResult {
    transform_for_target_with_filename(root, analysis, TransformTarget::Server, filename)
}

pub fn transform_for_target(
    root: &Root<'_>,
    analysis: &AnalysisTables,
    target: TransformTarget,
) -> TransformResult {
    transform_for_target_with_filename(root, analysis, target, None)
}

pub fn transform_for_target_with_filename(
    root: &Root<'_>,
    analysis: &AnalysisTables,
    target: TransformTarget,
    filename: Option<&str>,
) -> TransformResult {
    let (css, css_hash, css_scope) = match &root.css {
        Some(stylesheet) => {
            let css_hash_input = css_hash_input(stylesheet.content_styles, filename);
            let css_hash = hash(&css_hash_input);
            let css_scope = format!("svelte-{css_hash}");
            let css = css::render_stylesheet(stylesheet, analysis, &css_scope, &root.fragment);
            (Some(css), Some(css_hash), Some(css_scope))
        }
        None => (None, None, None),
    };

    let component = js::render_component(
        root,
        analysis,
        css_hash.as_deref(),
        css_scope.as_deref(),
        target,
    );
    let (runtime_specifier, runtime_source) = match target {
        TransformTarget::Server => (
            runtime::SERVER_RUNTIME_SPECIFIER,
            runtime::server_runtime_source(),
        ),
        TransformTarget::Client => (
            runtime::CLIENT_RUNTIME_SPECIFIER,
            runtime::client_runtime_source(),
        ),
    };
    let runtime_modules = if component.needs_runtime_import {
        vec![RuntimeModule {
            specifier: runtime_specifier.to_string(),
            code: runtime_source.to_string(),
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

fn css_hash_input(css: &str, filename: Option<&str>) -> String {
    match filename {
        Some("(unknown)") | None => css.to_string(),
        Some(filename) => filename.replace('\\', "/"),
    }
}
