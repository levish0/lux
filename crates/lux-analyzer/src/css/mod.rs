mod rule;
mod selector;

use lux_ast::analysis::AnalysisTables;
use lux_ast::css::StyleSheet;
use lux_ast::css::stylesheet::StyleSheetChild;

pub(super) fn analyze_stylesheet<'a>(stylesheet: &StyleSheet<'a>, tables: &mut AnalysisTables) {
    for child in &stylesheet.children {
        match child {
            StyleSheetChild::Rule(rule) => rule::analyze_rule(rule, None, tables),
            StyleSheetChild::Atrule(atrule) => rule::analyze_atrule(atrule, None, tables),
        }
    }
}
