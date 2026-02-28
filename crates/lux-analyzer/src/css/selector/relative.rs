use lux_ast::analysis::{
    AnalysisNodeKind, AnalysisTables, ComplexSelectorAnalysis, RelativeSelectorAnalysis, SpanKey,
};
use lux_ast::css::selector::{ComplexSelector, RelativeSelector, SimpleSelector};

use super::global::is_relative_global;

pub(super) fn analyze_relative_selector(
    relative_selector: &RelativeSelector<'_>,
    force_global_like: bool,
    tables: &mut AnalysisTables,
) -> RelativeSelectorAnalysis {
    let is_global =
        !relative_selector.selectors.is_empty() && is_relative_global(relative_selector);
    let mut is_global_like = force_global_like || is_view_transition_or_host(relative_selector);

    let has_root = relative_selector.selectors.iter().any(|selector| {
        matches!(
            selector,
            SimpleSelector::PseudoClassSelector(pseudo_class) if pseudo_class.name == "root"
        )
    });
    let has_has = relative_selector.selectors.iter().any(|selector| {
        matches!(
            selector,
            SimpleSelector::PseudoClassSelector(pseudo_class) if pseudo_class.name == "has"
        )
    });

    if has_root && !has_has {
        is_global_like = true;
    }

    if is_global || is_global_like {
        mark_nested_complex_selectors_used(relative_selector, tables);
    }

    RelativeSelectorAnalysis {
        is_global,
        is_global_like,
        scoped: !(is_global || is_global_like),
    }
}

pub(super) fn insert_relative_analysis(
    relative_selector: &RelativeSelector<'_>,
    analysis: RelativeSelectorAnalysis,
    tables: &mut AnalysisTables,
) {
    let key = SpanKey::new(AnalysisNodeKind::RelativeSelector, relative_selector.span);
    tables.relative_selectors.insert(key, analysis);
}

pub(super) fn insert_complex_analysis(
    complex_selector: &ComplexSelector<'_>,
    is_global: bool,
    tables: &mut AnalysisTables,
) {
    let key = SpanKey::new(AnalysisNodeKind::ComplexSelector, complex_selector.span);
    tables
        .complex_selectors
        .entry(key)
        .and_modify(|existing| {
            existing.is_global = is_global;
            existing.used |= is_global;
        })
        .or_insert(ComplexSelectorAnalysis {
            is_global,
            used: is_global,
        });
}

fn mark_nested_complex_selectors_used(
    relative_selector: &RelativeSelector<'_>,
    tables: &mut AnalysisTables,
) {
    for selector in &relative_selector.selectors {
        mark_nested_complex_selectors_used_in_selector(selector, tables);
    }
}

fn mark_nested_complex_selectors_used_in_selector(
    selector: &SimpleSelector<'_>,
    tables: &mut AnalysisTables,
) {
    let SimpleSelector::PseudoClassSelector(pseudo_class) = selector else {
        return;
    };
    let Some(args) = &pseudo_class.args else {
        return;
    };

    for complex_selector in &args.children {
        let key = SpanKey::new(AnalysisNodeKind::ComplexSelector, complex_selector.span);
        tables.complex_selectors.entry(key).or_default().used = true;

        for relative_selector in &complex_selector.children {
            for nested_selector in &relative_selector.selectors {
                mark_nested_complex_selectors_used_in_selector(nested_selector, tables);
            }
        }
    }
}

fn is_view_transition_or_host(relative_selector: &RelativeSelector<'_>) -> bool {
    if relative_selector.selectors.is_empty() {
        return false;
    }

    if !relative_selector.selectors.iter().all(|selector| {
        matches!(
            selector,
            SimpleSelector::PseudoClassSelector(_) | SimpleSelector::PseudoElementSelector(_)
        )
    }) {
        return false;
    }

    match &relative_selector.selectors[0] {
        SimpleSelector::PseudoClassSelector(pseudo_class) => pseudo_class.name == "host",
        SimpleSelector::PseudoElementSelector(pseudo_element) => matches!(
            pseudo_element.name,
            "view-transition"
                | "view-transition-group"
                | "view-transition-old"
                | "view-transition-new"
                | "view-transition-image-pair"
        ),
        _ => false,
    }
}
