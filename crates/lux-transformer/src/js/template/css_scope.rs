use lux_ast::analysis::{AnalysisNodeKind, AnalysisTables, RelativeSelectorAnalysis, SpanKey};
use lux_ast::css::selector::{ClassSelector, RelativeSelector, SelectorList, SimpleSelector};
use lux_ast::css::stylesheet::{
    CssAtrule, CssBlock, CssBlockChild, CssRule, StyleSheet, StyleSheetChild,
};
use lux_ast::template::attribute::{AttributeNode, AttributeValue};
use lux_ast::template::tag::TextOrExpressionTag;

use super::StaticRenderContext;

pub(super) fn scope_class_for_element<'a>(
    context: &StaticRenderContext<'a>,
    name: &str,
    attributes: &[AttributeNode<'a>],
) -> Option<&'a str> {
    let css_scope = context.css_scope?;
    let stylesheet = context.stylesheet?;

    stylesheet_targets_element(stylesheet, context.analysis, name, attributes).then_some(css_scope)
}

fn stylesheet_targets_element(
    stylesheet: &StyleSheet<'_>,
    analysis: &AnalysisTables,
    name: &str,
    attributes: &[AttributeNode<'_>],
) -> bool {
    stylesheet
        .children
        .iter()
        .any(|child| style_child_targets_element(child, analysis, name, attributes))
}

fn style_child_targets_element(
    child: &StyleSheetChild<'_>,
    analysis: &AnalysisTables,
    name: &str,
    attributes: &[AttributeNode<'_>],
) -> bool {
    match child {
        StyleSheetChild::Rule(rule) => rule_targets_element(rule, analysis, name, attributes),
        StyleSheetChild::Atrule(atrule) => {
            atrule_targets_element(atrule, analysis, name, attributes)
        }
    }
}

fn rule_targets_element(
    rule: &CssRule<'_>,
    analysis: &AnalysisTables,
    name: &str,
    attributes: &[AttributeNode<'_>],
) -> bool {
    selector_list_targets_element(&rule.prelude, analysis, name, attributes)
        || block_targets_element(&rule.block, analysis, name, attributes)
}

fn atrule_targets_element(
    atrule: &CssAtrule<'_>,
    analysis: &AnalysisTables,
    name: &str,
    attributes: &[AttributeNode<'_>],
) -> bool {
    atrule
        .block
        .as_ref()
        .is_some_and(|block| block_targets_element(block, analysis, name, attributes))
}

fn block_targets_element(
    block: &CssBlock<'_>,
    analysis: &AnalysisTables,
    name: &str,
    attributes: &[AttributeNode<'_>],
) -> bool {
    block.children.iter().any(|child| match child {
        CssBlockChild::Declaration(_) => false,
        CssBlockChild::Rule(rule) => rule_targets_element(rule, analysis, name, attributes),
        CssBlockChild::Atrule(atrule) => atrule_targets_element(atrule, analysis, name, attributes),
    })
}

fn selector_list_targets_element(
    selector_list: &SelectorList<'_>,
    analysis: &AnalysisTables,
    name: &str,
    attributes: &[AttributeNode<'_>],
) -> bool {
    selector_list.children.iter().any(|complex| {
        complex.children.iter().any(|relative| {
            relative_analysis(relative, analysis).scoped
                && relative_targets_element(relative, analysis, name, attributes)
        })
    })
}

fn relative_targets_element(
    relative: &RelativeSelector<'_>,
    analysis: &AnalysisTables,
    name: &str,
    attributes: &[AttributeNode<'_>],
) -> bool {
    for selector in &relative.selectors {
        if !simple_selector_targets_element(selector, analysis, name, attributes) {
            return false;
        }
    }

    true
}

fn simple_selector_targets_element(
    selector: &SimpleSelector<'_>,
    analysis: &AnalysisTables,
    name: &str,
    attributes: &[AttributeNode<'_>],
) -> bool {
    match selector {
        SimpleSelector::TypeSelector(type_selector) => {
            type_selector.name == "*" || type_selector.name.eq_ignore_ascii_case(name)
        }
        SimpleSelector::IdSelector(id_selector) => {
            static_attribute_value(attributes, "id").is_some_and(|value| value == id_selector.name)
        }
        SimpleSelector::ClassSelector(class_selector) => {
            element_has_class(attributes, class_selector)
        }
        SimpleSelector::AttributeSelector(attribute_selector) => {
            let Some(value) = static_attribute_value(attributes, attribute_selector.name) else {
                return has_boolean_attribute(attributes, attribute_selector.name);
            };

            attribute_selector
                .value
                .is_none_or(|expected| value == strip_attribute_selector_quotes(expected))
        }
        SimpleSelector::PseudoClassSelector(pseudo_class) => match pseudo_class.name {
            "global" | "root" | "host" => false,
            "is" | "where" => pseudo_class
                .args
                .as_ref()
                .is_none_or(|args| selector_list_targets_element(args, analysis, name, attributes)),
            "not" => pseudo_class.args.as_ref().is_none_or(|args| {
                !selector_list_targets_element(args, analysis, name, attributes)
            }),
            _ => true,
        },
        SimpleSelector::PseudoElementSelector(_)
        | SimpleSelector::Percentage(_)
        | SimpleSelector::Nth(_)
        | SimpleSelector::NestingSelector(_) => true,
    }
}

fn relative_analysis(
    relative: &RelativeSelector<'_>,
    analysis: &AnalysisTables,
) -> RelativeSelectorAnalysis {
    analysis
        .relative_selectors
        .get(&SpanKey::new(
            AnalysisNodeKind::RelativeSelector,
            relative.span,
        ))
        .cloned()
        .unwrap_or(RelativeSelectorAnalysis {
            is_global: false,
            is_global_like: false,
            scoped: false,
        })
}

fn element_has_class(attributes: &[AttributeNode<'_>], selector: &ClassSelector<'_>) -> bool {
    attributes.iter().any(|attribute| match attribute {
        AttributeNode::Attribute(attribute) if attribute.name == "class" => {
            static_class_tokens(&attribute.value)
                .iter()
                .any(|token| *token == selector.name)
        }
        AttributeNode::ClassDirective(directive) => directive.name == selector.name,
        _ => false,
    })
}

fn static_attribute_value<'a>(attributes: &'a [AttributeNode<'a>], name: &str) -> Option<&'a str> {
    attributes.iter().find_map(|attribute| {
        let AttributeNode::Attribute(attribute) = attribute else {
            return None;
        };
        if attribute.name != name {
            return None;
        }
        match &attribute.value {
            AttributeValue::True => Some(""),
            AttributeValue::ExpressionTag(_) => None,
            AttributeValue::Sequence(chunks) => static_text_chunks(chunks),
        }
    })
}

fn has_boolean_attribute(attributes: &[AttributeNode<'_>], name: &str) -> bool {
    attributes.iter().any(|attribute| {
        matches!(attribute, AttributeNode::Attribute(attribute) if attribute.name == name)
    })
}

fn static_class_tokens<'a>(value: &'a AttributeValue<'a>) -> Vec<&'a str> {
    match value {
        AttributeValue::True | AttributeValue::ExpressionTag(_) => Vec::new(),
        AttributeValue::Sequence(chunks) => static_text_chunks(chunks)
            .map(|text| text.split_whitespace().collect::<Vec<_>>())
            .unwrap_or_default(),
    }
}

fn static_text_chunks<'a>(chunks: &'a [TextOrExpressionTag<'a>]) -> Option<&'a str> {
    if chunks
        .iter()
        .any(|chunk| matches!(chunk, TextOrExpressionTag::ExpressionTag(_)))
    {
        return None;
    }

    if chunks.len() != 1 {
        return None;
    }

    match &chunks[0] {
        TextOrExpressionTag::Text(text) => Some(text.raw),
        TextOrExpressionTag::ExpressionTag(_) => None,
    }
}

fn strip_attribute_selector_quotes(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .or_else(|| {
            value
                .strip_prefix('\'')
                .and_then(|rest| rest.strip_suffix('\''))
        })
        .unwrap_or(value)
}
