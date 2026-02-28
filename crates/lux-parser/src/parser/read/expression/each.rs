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

    let result = OxcParser::new(allocator, remaining, source_type).parse_expression();

    match result {
        Ok(expression) => {
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

            let consumed = expression.span().end as usize;
            let _ = take(consumed).parse_next(input)?;
            let mut expression = expression;
            shift_expression_spans(&mut expression, offset);
            Ok(expression)
        }
        Err(_) => {
            let end = scan_each_expression_boundary(remaining).ok_or_else(ContextError::new)?;
            let expression_source = remaining[..end].trim_end();

            if expression_source.is_empty() {
                return Err(ContextError::new());
            }

            let mut expression = OxcParser::new(allocator, expression_source, source_type)
                .parse_expression()
                .map_err(|_| ContextError::new())?;
            let _ = take(expression_source.len()).parse_next(input)?;
            shift_expression_spans(&mut expression, offset);
            Ok(expression)
        }
    }
}
