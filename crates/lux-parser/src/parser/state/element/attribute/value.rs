use lux_ast::common::Span;
use lux_ast::template::attribute::AttributeValue;
use lux_ast::template::tag::{ExpressionTag, Text, TextOrExpressionTag};
use winnow::Result;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::{any, literal, take_while};

use crate::input::Input;
use crate::parser::read::expression::read_expression;
use crate::parser::utils::helpers::skip_whitespace;

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

    // Single expression without surrounding text → ExpressionTag
    if chunks.len() == 1 {
        if matches!(chunks.first(), Some(TextOrExpressionTag::ExpressionTag(_))) {
            let chunk = chunks.into_iter().next().unwrap();
            match chunk {
                TextOrExpressionTag::ExpressionTag(et) => {
                    return Ok(AttributeValue::ExpressionTag(et));
                }
                _ => unreachable!(),
            }
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
        metadata: None,
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
    let text = Text {
        span: Span::new(start as u32, end as u32),
        data: value,
        raw: value,
    };
    Ok(AttributeValue::Sequence(vec![TextOrExpressionTag::Text(
        text,
    )]))
}

fn read_sequence<'a>(input: &mut Input<'a>, quote: u8) -> Result<Vec<TextOrExpressionTag<'a>>> {
    let template = input.state.template;
    let mut chunks: Vec<TextOrExpressionTag<'a>> = Vec::new();
    let mut text_start: Option<usize> = None;

    loop {
        let remaining: &str = &input.input;
        if remaining.is_empty() {
            break;
        }

        let b = remaining.as_bytes()[0];

        if b == quote {
            if let Some(ts) = text_start.take() {
                let end = input.current_token_start();
                let text_slice = &template[ts..end];
                chunks.push(TextOrExpressionTag::Text(Text {
                    span: Span::new(ts as u32, end as u32),
                    data: text_slice,
                    raw: text_slice,
                }));
            }
            break;
        }

        if b == b'{' {
            if let Some(ts) = text_start.take() {
                let end = input.current_token_start();
                let text_slice = &template[ts..end];
                chunks.push(TextOrExpressionTag::Text(Text {
                    span: Span::new(ts as u32, end as u32),
                    data: text_slice,
                    raw: text_slice,
                }));
            }

            let expr_start = input.current_token_start();
            literal("{").parse_next(input)?;
            skip_whitespace(input);
            let expression = read_expression(input)?;
            skip_whitespace(input);
            literal("}").parse_next(input)?;
            let expr_end = input.previous_token_end();

            chunks.push(TextOrExpressionTag::ExpressionTag(ExpressionTag {
                span: Span::new(expr_start as u32, expr_end as u32),
                expression,
                metadata: None,
            }));
        } else {
            if text_start.is_none() {
                text_start = Some(input.current_token_start());
            }
            let _: char = any.parse_next(input)?;
        }
    }

    Ok(chunks)
}
