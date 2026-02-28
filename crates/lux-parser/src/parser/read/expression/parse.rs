use crate::input::Input;
use crate::parser::utils::scanner::scan_expression_boundary;
use crate::parser::utils::span_offset::shift_expression_spans;
use oxc_ast::ast::Expression;
use oxc_parser::Parser as OxcParser;
use oxc_span::GetSpan;
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
        Err(_) => Err(ContextError::new()),
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
        return Err(ContextError::new());
    }

    let source_type = make_source_type(ts);
    let result = OxcParser::new(allocator, expression_source, source_type).parse_expression();

    match result {
        Ok(mut expression) => {
            let _ = take(expression_source.len()).parse_next(input)?;
            shift_expression_spans(&mut expression, offset);
            Ok(expression)
        }
        Err(_) => Err(ContextError::new()),
    }
}
