use std::borrow::Cow;

use lux_ast::common::Span;
use lux_ast::template::attribute::AttributeValue;
use lux_ast::template::tag::{ExpressionTag, Text, TextOrExpressionTag};
use lux_utils::html_entities::decode_character_references;
use winnow::Result;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::{any, literal, take_while};

use crate::input::Input;
use crate::parser::read::expression::read_expression;
use crate::parser::utils::helpers::skip_whitespace;

fn decode_attr_text<'a>(
    raw: &'a str,
    span: Span,
    allocator: &'a oxc_allocator::Allocator,
) -> Text<'a> {
    let decoded = decode_character_references(raw, true);
    let data = match decoded {
        Cow::Borrowed(_) => raw,
        Cow::Owned(s) => &*allocator.alloc_str(&s),
    };
    Text { span, data, raw }
}

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
    let span = Span::new(start as u32, end as u32);
    let text = decode_attr_text(value, span, input.state.allocator);
    Ok(AttributeValue::Sequence(vec![TextOrExpressionTag::Text(
        text,
    )]))
}

fn read_sequence<'a>(input: &mut Input<'a>, quote: u8) -> Result<Vec<TextOrExpressionTag<'a>>> {
    let template = input.state.template;
    let allocator = input.state.allocator;
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
                let span = Span::new(ts as u32, end as u32);
                chunks.push(TextOrExpressionTag::Text(decode_attr_text(
                    text_slice, span, allocator,
                )));
            }
            break;
        }

        if b == b'{' {
            if let Some(ts) = text_start.take() {
                let end = input.current_token_start();
                let text_slice = &template[ts..end];
                let span = Span::new(ts as u32, end as u32);
                chunks.push(TextOrExpressionTag::Text(decode_attr_text(
                    text_slice, span, allocator,
                )));
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
