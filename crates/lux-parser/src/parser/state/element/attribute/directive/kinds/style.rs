use lux_ast::common::Span;
use lux_ast::template::attribute::{AttributeNode, AttributeValue};
use lux_ast::template::directive::{StyleDirective, StyleDirectiveValue};
use winnow::Result;

use super::super::modifier::parse_style_modifier;

pub(super) fn parse_style<'a>(
    name: &'a str,
    value: Option<AttributeValue<'a>>,
    modifiers_str: Vec<&'a str>,
    span: Span,
) -> Result<AttributeNode<'a>> {
    let style_value = match value {
        Some(AttributeValue::True) | None => StyleDirectiveValue::True,
        Some(AttributeValue::ExpressionTag(tag)) => StyleDirectiveValue::ExpressionTag(tag),
        Some(AttributeValue::Sequence(sequence)) => StyleDirectiveValue::Sequence(sequence),
    };

    let modifiers = modifiers_str
        .iter()
        .filter_map(|modifier| parse_style_modifier(modifier))
        .collect();

    Ok(AttributeNode::StyleDirective(StyleDirective {
        span,
        name,
        value: style_value,
        modifiers,
    }))
}
