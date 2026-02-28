use crate::input::Input;
use crate::parser::utils::scanner::{scan_each_expression_boundary, scan_expression_boundary};
use crate::parser::utils::span_offset::shift_expression_spans;
use oxc_ast::ast::Expression;
use oxc_parser::Parser as OxcParser;
use oxc_span::{GetSpan, SourceType};
use winnow::Result;
use winnow::error::ContextError;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::take;

fn make_source_type(ts: bool) -> SourceType {
    if ts {
        SourceType::ts()
    } else {
        SourceType::mjs()
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oxc_grammar_boundary() {
        use oxc_allocator::Allocator;
        use oxc_parser::Parser as OxcParser;

        let allocator = Allocator::default();

        // OXC stops at expression boundary - grammar-based, not byte scanning
        let r = OxcParser::new(&allocator, "x + 1} rest", SourceType::mjs()).parse_expression();
        assert!(r.is_ok());
        assert_eq!(r.unwrap().span().end, 5);

        let r = OxcParser::new(&allocator, "foo(1, 2)} more", SourceType::mjs()).parse_expression();
        assert!(r.is_ok());
        assert_eq!(r.unwrap().span().end, 9);

        let r = OxcParser::new(&allocator, "{ a: 1 }} rest", SourceType::mjs()).parse_expression();
        assert!(r.is_ok());
        assert_eq!(r.unwrap().span().end, 8);

        // Regex with } in char class - grammar handles correctly
        let r =
            OxcParser::new(&allocator, "/[}]/.test(x)} rest", SourceType::mjs()).parse_expression();
        assert!(r.is_ok());
        assert_eq!(r.unwrap().span().end, 13);

        // TS as expression consumed by OXC
        let r = OxcParser::new(&allocator, "items as item}", SourceType::ts()).parse_expression();
        assert!(r.is_ok());
        assert_eq!(r.unwrap().span().end, 13);

        // JS mode with `as` - OXC fails (needs fallback)
        let r = OxcParser::new(&allocator, "items as item}", SourceType::mjs()).parse_expression();
        assert!(r.is_err());
    }
}
