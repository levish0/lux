use lux_ast::common::Span;
use lux_ast::template::attribute::{AttributeNode, AttributeValue};
use lux_ast::template::directive::{AnimateDirective, LetDirective, UseDirective};
use winnow::Result;

use super::super::extract::extract_expression_from_value;

pub(super) fn parse_use<'a>(
    name: &'a str,
    value: Option<AttributeValue<'a>>,
    span: Span,
) -> Result<AttributeNode<'a>> {
    let expression = value.map(extract_expression_from_value).transpose()?;
    Ok(AttributeNode::UseDirective(UseDirective {
        span,
        name,
        expression,
    }))
}

pub(super) fn parse_let<'a>(
    name: &'a str,
    value: Option<AttributeValue<'a>>,
    span: Span,
) -> Result<AttributeNode<'a>> {
    let expression = value.map(extract_expression_from_value).transpose()?;
    Ok(AttributeNode::LetDirective(LetDirective {
        span,
        name,
        expression,
    }))
}

pub(super) fn parse_animate<'a>(
    name: &'a str,
    value: Option<AttributeValue<'a>>,
    span: Span,
) -> Result<AttributeNode<'a>> {
    let expression = value.map(extract_expression_from_value).transpose()?;
    Ok(AttributeNode::AnimateDirective(AnimateDirective {
        span,
        name,
        expression,
    }))
}
