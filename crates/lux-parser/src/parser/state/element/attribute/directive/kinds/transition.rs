use lux_ast::common::Span;
use lux_ast::template::attribute::{AttributeNode, AttributeValue};
use lux_ast::template::directive::TransitionDirective;
use winnow::Result;

use super::super::extract::extract_expression_from_value;
use super::super::modifier::parse_transition_modifier;

pub(super) fn parse_transition<'a>(
    name: &'a str,
    value: Option<AttributeValue<'a>>,
    modifiers_str: Vec<&'a str>,
    span: Span,
    intro: bool,
    outro: bool,
) -> Result<AttributeNode<'a>> {
    let expression = value.map(extract_expression_from_value).transpose()?;
    let modifiers = modifiers_str
        .iter()
        .filter_map(|modifier| parse_transition_modifier(modifier))
        .collect();

    Ok(AttributeNode::TransitionDirective(TransitionDirective {
        span,
        name,
        expression,
        modifiers,
        intro,
        outro,
    }))
}
