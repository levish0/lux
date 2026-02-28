use lux_ast::common::Span;
use lux_ast::template::tag::{ExpressionTag, TextOrExpressionTag};
use winnow::Result;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::{any, literal};

use crate::input::Input;
use crate::parser::read::expression::read_expression;
use crate::parser::utils::helpers::skip_whitespace;

use super::decode::decode_attr_text;

pub(super) fn read_sequence<'a>(
    input: &mut Input<'a>,
    quote: u8,
) -> Result<Vec<TextOrExpressionTag<'a>>> {
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
            flush_text_chunk(input, &mut chunks, &mut text_start, template, allocator);
            break;
        }

        if b == b'{' {
            flush_text_chunk(input, &mut chunks, &mut text_start, template, allocator);

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

fn flush_text_chunk<'a>(
    input: &mut Input<'a>,
    chunks: &mut Vec<TextOrExpressionTag<'a>>,
    text_start: &mut Option<usize>,
    template: &'a str,
    allocator: &'a oxc_allocator::Allocator,
) {
    let Some(start) = text_start.take() else {
        return;
    };

    let end = input.current_token_start();
    let text_slice = &template[start..end];
    let span = Span::new(start as u32, end as u32);
    chunks.push(TextOrExpressionTag::Text(decode_attr_text(
        text_slice, span, allocator,
    )));
}
