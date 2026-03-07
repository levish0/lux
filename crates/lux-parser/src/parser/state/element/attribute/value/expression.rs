use lux_ast::common::Span;
use lux_ast::template::attribute::AttributeValue;
use lux_ast::template::tag::ExpressionTag;
use winnow::Result;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::literal;

use crate::input::Input;
use crate::parser::read::expression::read_expression_until;
use crate::parser::utils::helpers::skip_whitespace;

pub(super) fn parse_expression_value<'a>(input: &mut Input<'a>) -> Result<AttributeValue<'a>> {
    let start = input.current_token_start();
    literal("{").parse_next(input)?;
    skip_whitespace(input);
    let expression = read_expression_until(input, b"")?;
    skip_whitespace(input);
    literal("}").parse_next(input)?;
    let end = input.previous_token_end();

    Ok(AttributeValue::ExpressionTag(ExpressionTag {
        span: Span::new(start as u32, end as u32),
        expression,
    }))
}
