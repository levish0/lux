use crate::input::Input;
use crate::parser::utils::scanner::{scan_await_expression_boundary, scan_expression_boundary};
use crate::parser::utils::span_offset::shift_expression_spans;
use oxc_allocator::Allocator;
use oxc_ast::AstBuilder;
use oxc_ast::ast::Expression;
use oxc_parser::Parser as OxcParser;
use oxc_span::GetSpan;
use oxc_span::Span;
use winnow::Result;
use winnow::error::ContextError;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::take;

use super::source_type::make_source_type;

/// Read a JS/TS expression using OXC's grammar-based parser.
/// OXC parses the remaining input and stops at the expression boundary,
/// similar to Acorn's `parseExpressionAt`.
pub fn read_expression<'a>(input: &mut Input<'a>) -> Result<Expression<'a>> {
    let offset = input.current_token_start() as u32;
    let allocator = input.state.allocator;
    let ts = input.state.ts;
    let remaining: &str = &input.input;

    if remaining.trim_start().is_empty() {
        return Err(ContextError::new());
    }

    let source_type = make_source_type(ts);
    let result = OxcParser::new(allocator, remaining, source_type).parse_expression();

    match result {
        Ok(mut expression) => {
            let consumed = expression.span().end as usize;
            let _ = take(consumed).parse_next(input)?;
            shift_expression_spans(&mut expression, offset);
            Ok(expression)
        }
        Err(_) => recover_expression_at_boundary(input, scan_expression_boundary),
    }
}

/// Read a JS/TS expression, stopping at template-level separator bytes at depth 0.
///
/// Used for delimiters that aren't JS operators but have meaning in the template:
/// - `=` in `{@const id = init}`
/// - `,` / `)` in `{#snippet name(a, b)}`
/// - `,` / `(` in `{#each expr as ctx, i (key)}`
pub fn read_expression_until<'a>(
    input: &mut Input<'a>,
    extra_stops: &[u8],
) -> Result<Expression<'a>> {
    let offset = input.current_token_start() as u32;
    let allocator = input.state.allocator;
    let ts = input.state.ts;
    let remaining: &str = &input.input;

    let end = scan_expression_boundary(remaining, extra_stops).ok_or_else(ContextError::new)?;
    let expression_source = remaining[..end].trim_end();

    if expression_source.is_empty() {
        return recover_expression(input, 0);
    }

    match parse_exact_expression(allocator, expression_source, ts) {
        Some(mut expression) => {
            let _ = take(expression_source.len()).parse_next(input)?;
            shift_expression_spans(&mut expression, offset);
            Ok(expression)
        }
        None => recover_expression(input, expression_source.len()),
    }
}

pub fn read_await_expression<'a>(input: &mut Input<'a>) -> Result<Expression<'a>> {
    let offset = input.current_token_start() as u32;
    let allocator = input.state.allocator;
    let ts = input.state.ts;
    let remaining: &str = &input.input;

    let end = scan_await_expression_boundary(remaining).ok_or_else(ContextError::new)?;
    let expression_source = remaining[..end].trim_end();

    if expression_source.is_empty() {
        return recover_expression(input, 0);
    }

    match parse_exact_expression(allocator, expression_source, ts) {
        Some(mut expression) => {
            let _ = take(expression_source.len()).parse_next(input)?;
            shift_expression_spans(&mut expression, offset);
            Ok(expression)
        }
        None => recover_expression(input, expression_source.len()),
    }
}

pub(crate) fn parse_exact_expression<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    ts: bool,
) -> Option<Expression<'a>> {
    let source_type = make_source_type(ts);
    let expression = OxcParser::new(allocator, source, source_type)
        .parse_expression()
        .ok()?;

    if expression.span().end as usize == source.len() {
        Some(expression)
    } else {
        None
    }
}

pub(crate) fn empty_identifier_expression<'a>(
    allocator: &'a Allocator,
    start: u32,
    end: u32,
) -> Expression<'a> {
    AstBuilder::new(allocator).expression_identifier(Span::new(start, end), "")
}

pub(crate) fn empty_identifier_reference<'a>(
    allocator: &'a Allocator,
    start: u32,
    end: u32,
) -> oxc_ast::ast::IdentifierReference<'a> {
    let Expression::Identifier(identifier) = empty_identifier_expression(allocator, start, end) else {
        unreachable!("empty identifier helper must return identifier");
    };
    identifier.unbox()
}

fn recover_expression_at_boundary<'a>(
    input: &mut Input<'a>,
    boundary: fn(&str, &[u8]) -> Option<usize>,
) -> Result<Expression<'a>> {
    let remaining: &str = &input.input;
    let Some(end) = boundary(remaining, b"") else {
        return Err(ContextError::new());
    };
    let expression_source = remaining[..end].trim_end();
    recover_expression(input, expression_source.len())
}

fn recover_expression<'a>(input: &mut Input<'a>, consumed: usize) -> Result<Expression<'a>> {
    if !input.state.loose {
        return Err(ContextError::new());
    }

    let start = input.current_token_start() as u32;
    if consumed > 0 {
        let _ = take(consumed).parse_next(input)?;
    }

    Ok(empty_identifier_expression(
        input.state.allocator,
        start,
        start + consumed as u32,
    ))
}
