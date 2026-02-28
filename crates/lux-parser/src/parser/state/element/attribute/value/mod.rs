use lux_ast::common::Span;
use lux_ast::template::attribute::AttributeValue;
use lux_ast::template::tag::ExpressionTag;
use winnow::Result;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::{any, literal, take_while};

use crate::input::Input;
use crate::parser::read::expression::read_expression;
use crate::parser::utils::helpers::skip_whitespace;

mod decode;
mod sequence;

use decode::decode_attr_text;
use sequence::read_sequence;

pub fn parse_attribute_value<'a>(input: &mut Input<'a>) -> Result<AttributeValue<'a>> {
    let remaining: &str = &input.input;
    let first = remaining.as_bytes().first().copied();

    match first {
        Some(b'"') | Some(b'\'') => parse_quoted_value(input, first.unwrap()),
        Some(b'{') => parse_expression_value(input),
        _ => parse_unquoted_value(input),
    }
}

fn parse_quoted_value<'a>(input: &mut Input<'a>, quote: u8) -> Result<AttributeValue<'a>> {
    let _: char = any.parse_next(input)?;

    let chunks = read_sequence(input, quote)?;

    // Consume closing quote
    let _: char = any.parse_next(input)?;

    // Single expression without surrounding text -> ExpressionTag
    if chunks.len() == 1
        && matches!(
            chunks.first(),
            Some(lux_ast::template::tag::TextOrExpressionTag::ExpressionTag(
                _
            ))
        )
    {
        let chunk = chunks.into_iter().next().expect("single item");
        match chunk {
            lux_ast::template::tag::TextOrExpressionTag::ExpressionTag(et) => {
                return Ok(AttributeValue::ExpressionTag(et));
            }
            _ => unreachable!(),
        }
    }

    Ok(AttributeValue::Sequence(chunks))
}

fn parse_expression_value<'a>(input: &mut Input<'a>) -> Result<AttributeValue<'a>> {
    let start = input.current_token_start();
    literal("{").parse_next(input)?;
    skip_whitespace(input);
    let expression = read_expression(input)?;
    skip_whitespace(input);
    literal("}").parse_next(input)?;
    let end = input.previous_token_end();

    Ok(AttributeValue::ExpressionTag(ExpressionTag {
        span: Span::new(start as u32, end as u32),
        expression,
    }))
}

fn parse_unquoted_value<'a>(input: &mut Input<'a>) -> Result<AttributeValue<'a>> {
    let value: &str = take_while(1.., |c: char| {
        !c.is_ascii_whitespace()
            && c != '>'
            && c != '/'
            && c != '"'
            && c != '\''
            && c != '='
            && c != '`'
    })
    .parse_next(input)?;

    let end = input.previous_token_end();
    let start = end - value.len();
    let span = Span::new(start as u32, end as u32);
    let text = decode_attr_text(value, span, input.state.allocator);

    Ok(AttributeValue::Sequence(vec![
        lux_ast::template::tag::TextOrExpressionTag::Text(text),
    ]))
}
