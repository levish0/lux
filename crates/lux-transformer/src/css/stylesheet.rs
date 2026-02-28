use lux_ast::analysis::AnalysisTables;
use lux_ast::css::StyleSheet;
use lux_ast::css::stylesheet::{
    CssAtrule, CssBlock, CssBlockChild, CssDeclaration, CssRule, StyleSheetChild,
};

use super::selector::render_selector_list;

pub(super) fn render(
    stylesheet: &StyleSheet<'_>,
    analysis: &AnalysisTables,
    scope_class: &str,
) -> String {
    stylesheet
        .children
        .iter()
        .map(|child| render_child(child, analysis, scope_class))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_child(
    child: &StyleSheetChild<'_>,
    analysis: &AnalysisTables,
    scope_class: &str,
) -> String {
    match child {
        StyleSheetChild::Rule(rule) => render_rule(rule, analysis, scope_class),
        StyleSheetChild::Atrule(atrule) => render_atrule(atrule, analysis, scope_class),
    }
}

fn render_rule(rule: &CssRule<'_>, analysis: &AnalysisTables, scope_class: &str) -> String {
    let selector = render_selector_list(&rule.prelude, analysis, scope_class);
    let block = render_block(&rule.block, analysis, scope_class);
    format!("{selector} {block}")
}

fn render_atrule(atrule: &CssAtrule<'_>, analysis: &AnalysisTables, scope_class: &str) -> String {
    if let Some(block) = &atrule.block {
        if atrule.prelude.is_empty() {
            format!(
                "@{} {}",
                atrule.name,
                render_block(block, analysis, scope_class)
            )
        } else {
            format!(
                "@{} {} {}",
                atrule.name,
                atrule.prelude,
                render_block(block, analysis, scope_class)
            )
        }
    } else if atrule.prelude.is_empty() {
        format!("@{};", atrule.name)
    } else {
        format!("@{} {};", atrule.name, atrule.prelude)
    }
}

fn render_block(block: &CssBlock<'_>, analysis: &AnalysisTables, scope_class: &str) -> String {
    if block.children.is_empty() {
        return "{}".to_string();
    }

    let body = block
        .children
        .iter()
        .map(|child| render_block_child(child, analysis, scope_class))
        .collect::<Vec<_>>()
        .join(" ");

    format!("{{ {body} }}")
}

fn render_block_child(
    child: &CssBlockChild<'_>,
    analysis: &AnalysisTables,
    scope_class: &str,
) -> String {
    match child {
        CssBlockChild::Declaration(declaration) => render_declaration(declaration),
        CssBlockChild::Rule(rule) => render_rule(rule, analysis, scope_class),
        CssBlockChild::Atrule(atrule) => render_atrule(atrule, analysis, scope_class),
    }
}

fn render_declaration(declaration: &CssDeclaration<'_>) -> String {
    format!("{}: {};", declaration.property, declaration.value)
}
