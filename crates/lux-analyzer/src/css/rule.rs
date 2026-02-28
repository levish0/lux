use lux_ast::analysis::{AnalysisNodeKind, AnalysisTables, CssRuleAnalysis, SpanKey};
use lux_ast::css::stylesheet::{CssAtrule, CssBlock, CssBlockChild, CssRule};

use super::selector::analyze_complex_selector;

pub(super) fn analyze_rule<'a>(
    rule: &CssRule<'a>,
    parent_rule: Option<SpanKey>,
    tables: &mut AnalysisTables,
) {
    let rule_key = SpanKey::new(AnalysisNodeKind::CssRule, rule.span);
    let mut rule_analysis = CssRuleAnalysis {
        parent_rule,
        ..CssRuleAnalysis::default()
    };

    for complex_selector in &rule.prelude.children {
        let (is_global, has_global_block) = analyze_complex_selector(complex_selector, tables);
        rule_analysis.has_global_selectors |= is_global;
        rule_analysis.has_local_selectors |= !is_global;
        rule_analysis.is_global_block |= has_global_block;
    }

    tables.css_rules.insert(rule_key, rule_analysis);
    analyze_block(&rule.block, Some(rule_key), tables);
}

pub(super) fn analyze_atrule<'a>(
    atrule: &CssAtrule<'a>,
    parent_rule: Option<SpanKey>,
    tables: &mut AnalysisTables,
) {
    if let Some(block) = &atrule.block {
        analyze_block(block, parent_rule, tables);
    }
}

fn analyze_block<'a>(
    block: &CssBlock<'a>,
    parent_rule: Option<SpanKey>,
    tables: &mut AnalysisTables,
) {
    for child in &block.children {
        match child {
            CssBlockChild::Declaration(_) => {}
            CssBlockChild::Rule(rule) => analyze_rule(rule, parent_rule, tables),
            CssBlockChild::Atrule(atrule) => analyze_atrule(atrule, parent_rule, tables),
        }
    }
}
