use crate::input::Input;
use crate::parser::utils::scanner::scan_each_expression_boundary;
use crate::parser::utils::span_offset::shift_expression_spans;
use oxc_ast::ast::Expression;
use oxc_parser::Parser as OxcParser;
use oxc_span::GetSpan;
use winnow::Result;
use winnow::error::ContextError;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::take;

use super::parse::{parse_exact_expression, empty_identifier_expression};
use super::source_type::make_source_type;

/// Read the each-block collection expression.
///
/// Strategy (mirrors Svelte's tag.js):
/// 1. Try OXC on the full remaining input (grammar-based).
/// 2. If OK: check for TS `as` consumption and unwrap if needed.
/// 3. If Err (JS mode, `as` confuses OXC): find ` as ` boundary, retry.
pub fn read_each_expression<'a>(input: &mut Input<'a>) -> Result<Expression<'a>> {
    let offset = input.current_token_start() as u32;
    let allocator = input.state.allocator;
    let ts = input.state.ts;
    let remaining: &str = &input.input;
    let source_type = make_source_type(ts);
    let boundary = scan_each_expression_boundary(remaining);

    let result = OxcParser::new(allocator, remaining, source_type).parse_expression();

    match result {
        Ok(expression) => {
            let consumed = expression.span().end as usize;
            if let Some(boundary) = boundary {
                let expression_source = remaining[..boundary].trim_end();
                if expression_source.is_empty() {
                    return recover_each_expression(input, 0);
                }

                if !expression_source.is_empty() && expression_source.len() < consumed {
                    return match parse_exact_expression(allocator, expression_source, ts) {
                        Some(mut expression) => {
                            let _ = take(expression_source.len()).parse_next(input)?;
                            shift_expression_spans(&mut expression, offset);
                            Ok(expression)
                        }
                        None => recover_each_expression(input, expression_source.len()),
                    };
                }

                if consumed < expression_source.len() {
                    let trailing = &expression_source[consumed..];
                    if !trailing.trim().is_empty() {
                        return recover_each_expression(input, expression_source.len());
                    }
                }
            }

            if ts && let Expression::TSAsExpression(ts_as) = &expression {
                let inner_end = ts_as.expression.span().end as usize;
                let _ = take(inner_end).parse_next(input)?;
                let inner_source = &remaining[..inner_end];
                let mut inner = OxcParser::new(allocator, inner_source, source_type)
                    .parse_expression()
                    .map_err(|_| ContextError::new())?;
                shift_expression_spans(&mut inner, offset);
                return Ok(inner);
            }

            let _ = take(consumed).parse_next(input)?;
            let mut expression = expression;
            shift_expression_spans(&mut expression, offset);
            Ok(expression)
        }
        Err(_) => {
            let end = boundary.ok_or_else(ContextError::new)?;
            let expression_source = remaining[..end].trim_end();

            if expression_source.is_empty() {
                return recover_each_expression(input, 0);
            }

            match parse_exact_expression(allocator, expression_source, ts) {
                Some(mut expression) => {
                    let _ = take(expression_source.len()).parse_next(input)?;
                    shift_expression_spans(&mut expression, offset);
                    Ok(expression)
                }
                None => recover_each_expression(input, expression_source.len()),
            }
        }
    }
}

fn recover_each_expression<'a>(input: &mut Input<'a>, consumed: usize) -> Result<Expression<'a>> {
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
