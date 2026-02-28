use lux_ast::common::Span;
use lux_ast::template::attribute::{AttributeNode, AttributeValue};
use lux_ast::template::directive::OnDirective;
use winnow::Result;

use super::super::extract::extract_expression_from_value;
use super::super::modifier::parse_event_modifier;

pub(super) fn parse_on<'a>(
    name: &'a str,
    value: Option<AttributeValue<'a>>,
    modifiers_str: Vec<&'a str>,
    span: Span,
) -> Result<AttributeNode<'a>> {
    let expression = value.map(extract_expression_from_value).transpose()?;
    let modifiers = modifiers_str
        .iter()
        .filter_map(|modifier| parse_event_modifier(modifier))
        .collect();

    Ok(AttributeNode::OnDirective(OnDirective {
        span,
        name,
        expression,
        modifiers,
    }))
}
