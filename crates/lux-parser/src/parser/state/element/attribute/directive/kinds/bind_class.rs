use lux_ast::common::Span;
use lux_ast::template::attribute::{AttributeNode, AttributeValue};
use lux_ast::template::directive::{BindDirective, ClassDirective};
use winnow::Result;

use crate::input::Input;

use super::super::extract::extract_directive_expression;

pub(super) fn parse_bind<'a>(
    input: &mut Input<'a>,
    name: &'a str,
    value: Option<AttributeValue<'a>>,
    span: Span,
) -> Result<AttributeNode<'a>> {
    let expression = extract_directive_expression(input, value, name)?;
    Ok(AttributeNode::BindDirective(BindDirective {
        span,
        name,
        expression,
    }))
}

pub(super) fn parse_class<'a>(
    input: &mut Input<'a>,
    name: &'a str,
    value: Option<AttributeValue<'a>>,
    span: Span,
) -> Result<AttributeNode<'a>> {
    let expression = extract_directive_expression(input, value, name)?;
    Ok(AttributeNode::ClassDirective(ClassDirective {
        span,
        name,
        expression,
    }))
}
