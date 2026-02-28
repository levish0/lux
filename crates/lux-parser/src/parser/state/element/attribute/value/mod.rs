use lux_ast::template::attribute::AttributeValue;
use winnow::Result;

use crate::input::Input;

mod decode;
mod expression;
mod quoted;
mod sequence;
mod unquoted;

use expression::parse_expression_value;
use quoted::parse_quoted_value;
use unquoted::parse_unquoted_value;

pub fn parse_attribute_value<'a>(input: &mut Input<'a>) -> Result<AttributeValue<'a>> {
    let remaining: &str = &input.input;
    let first = remaining.as_bytes().first().copied();

    match first {
        Some(b'"') | Some(b'\'') => parse_quoted_value(input, first.expect("quote byte exists")),
        Some(b'{') => parse_expression_value(input),
        _ => parse_unquoted_value(input),
    }
}
