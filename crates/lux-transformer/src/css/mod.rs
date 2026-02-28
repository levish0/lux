mod selector;
mod stylesheet;

use lux_ast::analysis::AnalysisTables;
use lux_ast::css::StyleSheet;

pub(super) fn render_stylesheet(
    stylesheet: &StyleSheet<'_>,
    analysis: &AnalysisTables,
    scope_class: &str,
) -> String {
    stylesheet::render(stylesheet, analysis, scope_class)
}
