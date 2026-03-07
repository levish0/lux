use lux_ast::analysis::AnalysisTables;
use lux_ast::css::StyleSheet;
use lux_ast::css::stylesheet::{
    CssAtrule, CssBlock, CssBlockChild, CssDeclaration, CssRule, StyleSheetChild,
};
use lux_ast::template::root::Fragment;

use super::CssOutputFormat;
use super::selector::render_selector_list;

pub(super) fn render(
    stylesheet: &StyleSheet<'_>,
    analysis: &AnalysisTables,
    scope_class: &str,
    fragment: &Fragment<'_>,
    format: CssOutputFormat,
) -> String {
    stylesheet
        .children
        .iter()
        .filter_map(|child| render_child(child, analysis, scope_class, fragment, format))
        .collect::<Vec<_>>()
        .join(match format {
            CssOutputFormat::External => "\n",
            CssOutputFormat::Embedded => "",
        })
}

fn render_child(
    child: &StyleSheetChild<'_>,
    analysis: &AnalysisTables,
    scope_class: &str,
    fragment: &Fragment<'_>,
    format: CssOutputFormat,
) -> Option<String> {
    match child {
        StyleSheetChild::Rule(rule) => render_rule(rule, analysis, scope_class, fragment, format),
        StyleSheetChild::Atrule(atrule) => {
            render_atrule(atrule, analysis, scope_class, fragment, format)
        }
    }
}

fn render_rule(
    rule: &CssRule<'_>,
    analysis: &AnalysisTables,
    scope_class: &str,
    fragment: &Fragment<'_>,
    format: CssOutputFormat,
) -> Option<String> {
    let selector = render_selector_list(&rule.prelude, analysis, scope_class, fragment);
    if selector.is_empty() {
        return None;
    }
    let block = render_block(&rule.block, analysis, scope_class, fragment, format);
    Some(format!("{selector} {block}"))
}

fn render_atrule(
    atrule: &CssAtrule<'_>,
    analysis: &AnalysisTables,
    scope_class: &str,
    fragment: &Fragment<'_>,
    format: CssOutputFormat,
) -> Option<String> {
    if let Some(block) = &atrule.block {
        if atrule.prelude.is_empty() {
            Some(format!(
                "@{}{}",
                atrule.name,
                render_block(block, analysis, scope_class, fragment, format)
            ))
        } else {
            Some(format!(
                "@{} {}{}",
                atrule.name,
                atrule.prelude,
                render_block(block, analysis, scope_class, fragment, format)
            ))
        }
    } else if atrule.prelude.is_empty() {
        Some(format!("@{};", atrule.name))
    } else {
        Some(format!("@{} {};", atrule.name, atrule.prelude))
    }
}

fn render_block(
    block: &CssBlock<'_>,
    analysis: &AnalysisTables,
    scope_class: &str,
    fragment: &Fragment<'_>,
    format: CssOutputFormat,
) -> String {
    if block.children.is_empty() {
        return "{}".to_string();
    }

    let body = block
        .children
        .iter()
        .map(|child| render_block_child(child, analysis, scope_class, fragment, format))
        .filter(|child| !child.is_empty())
        .collect::<Vec<_>>()
        .join(match format {
            CssOutputFormat::External => " ",
            CssOutputFormat::Embedded => "",
        });

    if body.is_empty() {
        return "{}".to_string();
    }

    match format {
        CssOutputFormat::External => format!("{{ {body} }}"),
        CssOutputFormat::Embedded => format!("{{{body}}}"),
    }
}

fn render_block_child(
    child: &CssBlockChild<'_>,
    analysis: &AnalysisTables,
    scope_class: &str,
    fragment: &Fragment<'_>,
    format: CssOutputFormat,
) -> String {
    match child {
        CssBlockChild::Declaration(declaration) => render_declaration(declaration, format),
        CssBlockChild::Rule(rule) => {
            render_rule(rule, analysis, scope_class, fragment, format).unwrap_or_default()
        }
        CssBlockChild::Atrule(atrule) => {
            render_atrule(atrule, analysis, scope_class, fragment, format).unwrap_or_default()
        }
    }
}

fn render_declaration(declaration: &CssDeclaration<'_>, format: CssOutputFormat) -> String {
    match format {
        CssOutputFormat::External => format!("{}: {};", declaration.property, declaration.value),
        CssOutputFormat::Embedded => format!("{}:{};", declaration.property, declaration.value),
    }
}
