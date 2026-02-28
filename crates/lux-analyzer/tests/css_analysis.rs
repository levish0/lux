use lux_analyzer::analyze;
use lux_ast::analysis::{AnalysisNodeKind, SpanKey};
use lux_ast::css::selector::SimpleSelector;
use lux_ast::css::stylesheet::StyleSheetChild;
use lux_parser::parse;
use oxc_allocator::Allocator;

#[test]
fn analyze_marks_global_selector_rule() {
    let allocator = Allocator::default();
    let result = parse(
        "<style>:global(body) { color: red; }</style>",
        &allocator,
        false,
    );
    assert!(result.errors.is_empty());

    let tables = analyze(&result.root);
    let css = result.root.css.as_ref().expect("expected stylesheet");
    let StyleSheetChild::Rule(rule) = &css.children[0] else {
        panic!("expected rule");
    };
    let complex = &rule.prelude.children[0];

    let rule_analysis = tables
        .css_rules
        .get(&SpanKey::new(AnalysisNodeKind::CssRule, rule.span))
        .expect("expected rule analysis");
    assert!(rule_analysis.has_global_selectors);
    assert!(!rule_analysis.has_local_selectors);
    assert!(!rule_analysis.is_global_block);

    let complex_analysis = tables
        .complex_selectors
        .get(&SpanKey::new(
            AnalysisNodeKind::ComplexSelector,
            complex.span,
        ))
        .expect("expected complex selector analysis");
    assert!(complex_analysis.is_global);
    assert!(complex_analysis.used);
}

#[test]
fn analyze_marks_global_block_descendants_as_global_like() {
    let allocator = Allocator::default();
    let result = parse(
        "<style>:global .x { color: red; }</style>",
        &allocator,
        false,
    );
    assert!(result.errors.is_empty());

    let tables = analyze(&result.root);
    let css = result.root.css.as_ref().expect("expected stylesheet");
    let StyleSheetChild::Rule(rule) = &css.children[0] else {
        panic!("expected rule");
    };
    let complex = &rule.prelude.children[0];
    assert_eq!(complex.children.len(), 2);

    let rule_analysis = tables
        .css_rules
        .get(&SpanKey::new(AnalysisNodeKind::CssRule, rule.span))
        .expect("expected rule analysis");
    assert!(rule_analysis.is_global_block);
    assert!(rule_analysis.has_global_selectors);
    assert!(!rule_analysis.has_local_selectors);

    let second_relative = &complex.children[1];
    let second_analysis = tables
        .relative_selectors
        .get(&SpanKey::new(
            AnalysisNodeKind::RelativeSelector,
            second_relative.span,
        ))
        .expect("expected relative selector analysis");
    assert!(second_analysis.is_global_like);
    assert!(!second_analysis.scoped);
}

#[test]
fn analyze_keeps_mixed_selector_local() {
    let allocator = Allocator::default();
    let result = parse(
        "<style>.a :global(.b) { color: red; }</style>",
        &allocator,
        false,
    );
    assert!(result.errors.is_empty());

    let tables = analyze(&result.root);
    let css = result.root.css.as_ref().expect("expected stylesheet");
    let StyleSheetChild::Rule(rule) = &css.children[0] else {
        panic!("expected rule");
    };
    let complex = &rule.prelude.children[0];
    assert_eq!(complex.children.len(), 2);

    let rule_analysis = tables
        .css_rules
        .get(&SpanKey::new(AnalysisNodeKind::CssRule, rule.span))
        .expect("expected rule analysis");
    assert!(rule_analysis.has_local_selectors);
    assert!(!rule_analysis.has_global_selectors);

    let complex_analysis = tables
        .complex_selectors
        .get(&SpanKey::new(
            AnalysisNodeKind::ComplexSelector,
            complex.span,
        ))
        .expect("expected complex selector analysis");
    assert!(!complex_analysis.is_global);

    let first_relative = &complex.children[0];
    let first_analysis = tables
        .relative_selectors
        .get(&SpanKey::new(
            AnalysisNodeKind::RelativeSelector,
            first_relative.span,
        ))
        .expect("expected first relative analysis");
    assert!(first_analysis.scoped);
}

#[test]
fn analyze_marks_nested_not_selector_as_used_under_root() {
    let allocator = Allocator::default();
    let result = parse(
        "<style>:root:not(.x) { color: red; }</style>",
        &allocator,
        false,
    );
    assert!(result.errors.is_empty());

    let tables = analyze(&result.root);
    let css = result.root.css.as_ref().expect("expected stylesheet");
    let StyleSheetChild::Rule(rule) = &css.children[0] else {
        panic!("expected rule");
    };
    let complex = &rule.prelude.children[0];
    let relative = &complex.children[0];

    let not_selector = relative
        .selectors
        .iter()
        .find_map(|selector| match selector {
            SimpleSelector::PseudoClassSelector(pseudo_class) if pseudo_class.name == "not" => {
                Some(pseudo_class)
            }
            _ => None,
        })
        .expect("expected :not pseudo class");

    let nested_complex = &not_selector
        .args
        .as_ref()
        .expect("expected :not args")
        .children[0];

    let nested_complex_analysis = tables
        .complex_selectors
        .get(&SpanKey::new(
            AnalysisNodeKind::ComplexSelector,
            nested_complex.span,
        ))
        .expect("expected nested complex selector analysis");
    assert!(nested_complex_analysis.used);
}

#[test]
fn analyze_treats_global_has_selector_as_scoped() {
    let allocator = Allocator::default();
    let result = parse(
        "<style>:global(.foo):has(.bar) { color: red; }</style>",
        &allocator,
        false,
    );
    assert!(result.errors.is_empty());

    let tables = analyze(&result.root);
    let css = result.root.css.as_ref().expect("expected stylesheet");
    let StyleSheetChild::Rule(rule) = &css.children[0] else {
        panic!("expected rule");
    };
    let complex = &rule.prelude.children[0];
    let relative = &complex.children[0];

    let rule_analysis = tables
        .css_rules
        .get(&SpanKey::new(AnalysisNodeKind::CssRule, rule.span))
        .expect("expected rule analysis");
    assert!(rule_analysis.has_local_selectors);
    assert!(!rule_analysis.has_global_selectors);

    let complex_analysis = tables
        .complex_selectors
        .get(&SpanKey::new(
            AnalysisNodeKind::ComplexSelector,
            complex.span,
        ))
        .expect("expected complex selector analysis");
    assert!(!complex_analysis.is_global);

    let relative_analysis = tables
        .relative_selectors
        .get(&SpanKey::new(
            AnalysisNodeKind::RelativeSelector,
            relative.span,
        ))
        .expect("expected relative analysis");
    assert!(!relative_analysis.is_global);
    assert!(relative_analysis.scoped);
}

#[test]
fn analyze_links_nested_rule_to_parent_rule() {
    let allocator = Allocator::default();
    let result = parse(
        "<style>.a { .b { color: red; } }</style>",
        &allocator,
        false,
    );
    assert!(result.errors.is_empty());

    let tables = analyze(&result.root);
    let css = result.root.css.as_ref().expect("expected stylesheet");
    let StyleSheetChild::Rule(outer_rule) = &css.children[0] else {
        panic!("expected outer rule");
    };

    let inner_rule = outer_rule
        .block
        .children
        .iter()
        .find_map(|child| match child {
            lux_ast::css::stylesheet::CssBlockChild::Rule(rule) => Some(rule),
            _ => None,
        })
        .expect("expected nested rule");

    let outer_key = SpanKey::new(AnalysisNodeKind::CssRule, outer_rule.span);
    let inner_key = SpanKey::new(AnalysisNodeKind::CssRule, inner_rule.span);

    let inner_analysis = tables
        .css_rules
        .get(&inner_key)
        .expect("expected nested rule analysis");
    assert_eq!(inner_analysis.parent_rule, Some(outer_key));
}
