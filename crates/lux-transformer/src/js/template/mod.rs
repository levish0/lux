mod attribute;
mod css_scope;
mod marker;
mod render;
mod runtime;

use lux_ast::analysis::AnalysisTables;
use lux_ast::css::stylesheet::StyleSheet;
use lux_ast::template::root::FragmentNode;
use oxc_ast::{AstBuilder, ast::Expression};
pub(crate) use runtime::RuntimeScope;

pub(super) struct TemplateRenderResult {
    pub html: String,
    pub has_dynamic: bool,
}

pub(super) struct StaticRenderContext<'a> {
    pub stylesheet: Option<&'a StyleSheet<'a>>,
    pub analysis: &'a AnalysisTables,
    pub css_scope: Option<&'a str>,
}

pub(super) fn render_nodes_template(
    nodes: &[&FragmentNode<'_>],
    context: &StaticRenderContext<'_>,
) -> TemplateRenderResult {
    let mut html = String::new();
    let mut has_dynamic = false;
    render::render_fragment_nodes(nodes, &mut html, &mut has_dynamic, context, None);
    TemplateRenderResult { html, has_dynamic }
}

pub(super) fn build_render_nodes_expression<'a>(
    ast: AstBuilder<'a>,
    nodes: &[&'a FragmentNode<'a>],
    scope: &RuntimeScope,
) -> Expression<'a> {
    runtime::build_render_nodes_expression(ast, nodes, scope)
}
