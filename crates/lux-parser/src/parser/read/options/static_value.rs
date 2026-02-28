use lux_ast::template::attribute::{Attribute, AttributeValue};
use lux_ast::template::tag::TextOrExpressionTag;
use oxc_ast::ast::Expression;

use crate::error::{ErrorKind, ParseError};

pub(super) enum StaticValue<'a> {
    String(&'a str),
    Bool(bool),
}

pub(super) fn get_static_value<'a>(attr: &Attribute<'a>) -> Result<StaticValue<'a>, ParseError> {
    match &attr.value {
        AttributeValue::True => Ok(StaticValue::Bool(true)),
        AttributeValue::ExpressionTag(tag) => {
            expression_to_static_value(tag.expression.get_inner_expression(), attr)
        }
        AttributeValue::Sequence(chunks) if chunks.len() == 1 => match &chunks[0] {
            TextOrExpressionTag::Text(t) => Ok(StaticValue::String(t.data)),
            TextOrExpressionTag::ExpressionTag(tag) => {
                expression_to_static_value(tag.expression.get_inner_expression(), attr)
            }
        },
        _ => Err(ParseError::with_code(
            ErrorKind::InvalidSvelteOptions,
            "svelte_options_invalid_attribute_value",
            attr.span,
            format!("{} must be a static value", attr.name),
        )),
    }
}

pub(super) fn get_static_string_value<'a>(attr: &Attribute<'a>) -> Result<&'a str, ParseError> {
    match get_static_value(attr)? {
        StaticValue::String(value) => Ok(value),
        StaticValue::Bool(_) => Err(ParseError::with_code(
            ErrorKind::InvalidSvelteOptions,
            "svelte_options_invalid_attribute_value",
            attr.span,
            format!("{} must be a string", attr.name),
        )),
    }
}

pub(super) fn get_boolean_value(attr: &Attribute<'_>) -> Result<bool, ParseError> {
    match get_static_value(attr)? {
        StaticValue::Bool(value) => Ok(value),
        _ => Err(ParseError::with_code(
            ErrorKind::InvalidSvelteOptions,
            "svelte_options_invalid_attribute_value",
            attr.span,
            format!("{} must be true or false", attr.name),
        )),
    }
}

fn expression_to_static_value<'a>(
    expression: &Expression<'a>,
    attr: &Attribute<'a>,
) -> Result<StaticValue<'a>, ParseError> {
    match expression {
        Expression::StringLiteral(string) => Ok(StaticValue::String(string.value.as_str())),
        Expression::BooleanLiteral(boolean) => Ok(StaticValue::Bool(boolean.value)),
        _ => Err(ParseError::with_code(
            ErrorKind::InvalidSvelteOptions,
            "svelte_options_invalid_attribute_value",
            attr.span,
            format!("{} must be a static value", attr.name),
        )),
    }
}
