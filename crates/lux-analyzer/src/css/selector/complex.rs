use lux_ast::analysis::AnalysisTables;
use lux_ast::css::selector::{ComplexSelector, SelectorList, SimpleSelector};

use super::global::starts_with_global_block;
use super::relative::{
    analyze_relative_selector, insert_complex_analysis, insert_relative_analysis,
};

pub(super) fn analyze_complex_selector(
    complex_selector: &ComplexSelector<'_>,
    tables: &mut AnalysisTables,
) -> (bool, bool) {
    analyze_nested_selector_lists(complex_selector, tables);

    let mut has_global_block = false;
    let mut in_global_block = false;
    let mut complex_is_global = true;

    for relative_selector in &complex_selector.children {
        let analysis = analyze_relative_selector(relative_selector, in_global_block, tables);
        insert_relative_analysis(relative_selector, analysis.clone(), tables);

        if starts_with_global_block(relative_selector) {
            has_global_block = true;
            in_global_block = true;
        }

        complex_is_global &= analysis.is_global || analysis.is_global_like;
    }

    insert_complex_analysis(complex_selector, complex_is_global, tables);
    (complex_is_global, has_global_block)
}

fn analyze_nested_selector_lists(
    complex_selector: &ComplexSelector<'_>,
    tables: &mut AnalysisTables,
) {
    for relative_selector in &complex_selector.children {
        for selector in &relative_selector.selectors {
            let SimpleSelector::PseudoClassSelector(pseudo_class) = selector else {
                continue;
            };
            let Some(args) = &pseudo_class.args else {
                continue;
            };
            analyze_selector_list(args, tables);
        }
    }
}

fn analyze_selector_list(selector_list: &SelectorList<'_>, tables: &mut AnalysisTables) {
    for complex_selector in &selector_list.children {
        analyze_complex_selector(complex_selector, tables);
    }
}
