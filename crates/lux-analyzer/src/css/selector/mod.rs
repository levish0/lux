mod complex;
mod global;
mod relative;

use lux_ast::analysis::AnalysisTables;
use lux_ast::css::selector::ComplexSelector;

pub(super) fn analyze_complex_selector(
    complex_selector: &ComplexSelector<'_>,
    tables: &mut AnalysisTables,
) -> (bool, bool) {
    complex::analyze_complex_selector(complex_selector, tables)
}
