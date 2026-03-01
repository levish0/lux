mod component;
mod template;

use lux_ast::analysis::AnalysisTables;
use lux_ast::template::root::Root;

pub(super) fn render_component(
    root: &Root<'_>,
    analysis: &AnalysisTables,
    css: Option<&str>,
    css_hash: Option<&str>,
    css_scope: Option<&str>,
) -> String {
    component::render(root, analysis, css, css_hash, css_scope)
}
