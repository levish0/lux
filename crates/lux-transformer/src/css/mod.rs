mod selector;
mod stylesheet;

use lux_ast::analysis::AnalysisTables;
use lux_ast::css::StyleSheet;
use lux_ast::template::root::Fragment;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CssOutputFormat {
    External,
    Embedded,
}

pub(super) fn render_stylesheet(
    stylesheet: &StyleSheet<'_>,
    analysis: &AnalysisTables,
    scope_class: &str,
    fragment: &Fragment<'_>,
) -> String {
    stylesheet::render(
        stylesheet,
        analysis,
        scope_class,
        fragment,
        CssOutputFormat::External,
    )
}

pub(super) fn render_stylesheet_embedded(
    stylesheet: &StyleSheet<'_>,
    analysis: &AnalysisTables,
    scope_class: &str,
    fragment: &Fragment<'_>,
) -> String {
    stylesheet::render(
        stylesheet,
        analysis,
        scope_class,
        fragment,
        CssOutputFormat::Embedded,
    )
}
