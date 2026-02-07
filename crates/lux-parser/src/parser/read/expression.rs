use oxc_ast::ast::Expression;
use oxc_parser::Parser as OxcParser;
use oxc_span::{GetSpan, SourceType};
use winnow::Result;
use winnow::error::ContextError;
use winnow::prelude::*;

use crate::input::Input;

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
    let allocator = input.state.allocator;
    let ts = input.state.ts;
    let remaining: &str = &input.input;

    if remaining.trim_start().is_empty() {
        return Err(ContextError::new());
    }

    let source_type = make_source_type(ts);
    let result = OxcParser::new(allocator, remaining, source_type).parse_expression();

    match result {
        Ok(expr) => {
            let end = expr.span().end as usize;
            let _ = winnow::token::take(end).parse_next(input)?;
            Ok(expr)
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
///
/// Finds the boundary via bracket-aware scanning, then validates with OXC.
pub fn read_expression_until<'a>(
    input: &mut Input<'a>,
    extra_stops: &[u8],
) -> Result<Expression<'a>> {
    let allocator = input.state.allocator;
    let ts = input.state.ts;

    let remaining: &str = &input.input;
    let end_offset = find_expression_end(remaining, extra_stops).ok_or_else(ContextError::new)?;

    let expr_str = remaining[..end_offset].trim_end();

    if expr_str.is_empty() {
        return Err(ContextError::new());
    }

    let source_type = make_source_type(ts);
    let result = OxcParser::new(allocator, expr_str, source_type).parse_expression();

    match result {
        Ok(expr) => {
            let _ = winnow::token::take(expr_str.len()).parse_next(input)?;
            Ok(expr)
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
    let allocator = input.state.allocator;
    let ts = input.state.ts;
    let remaining: &str = &input.input;

    let source_type = make_source_type(ts);

    // Primary: let OXC determine the expression boundary via grammar.
    let result = OxcParser::new(allocator, remaining, source_type).parse_expression();

    match result {
        Ok(expr) => {
            if ts {
                // In TS mode, OXC may have consumed ` as ` as TSAsExpression.
                // Check and unwrap if the outermost node is TSAsExpression.
                if let Expression::TSAsExpression(ts_as) = &expr {
                    let inner_end = ts_as.expression.span().end as usize;
                    let _ = winnow::token::take(inner_end).parse_next(input)?;
                    // Re-parse just the inner expression to get a clean AST node.
                    let inner_str = &remaining[..inner_end];
                    let inner = OxcParser::new(allocator, inner_str, source_type)
                        .parse_expression()
                        .map_err(|_| ContextError::new())?;
                    return Ok(inner);
                }
            }
            let end = expr.span().end as usize;
            let _ = winnow::token::take(end).parse_next(input)?;
            Ok(expr)
        }
        Err(_) => {
            // OXC failed — likely JS mode where `as` confuses the parser.
            // Find ` as ` or `}` boundary, then parse up to it.
            let boundary = find_each_expression_end(remaining).ok_or_else(ContextError::new)?;
            let expr_str = remaining[..boundary].trim_end();
            if expr_str.is_empty() {
                return Err(ContextError::new());
            }
            let expr = OxcParser::new(allocator, expr_str, source_type)
                .parse_expression()
                .map_err(|_| ContextError::new())?;
            let _ = winnow::token::take(expr_str.len()).parse_next(input)?;
            Ok(expr)
        }
    }
}

/// Find end of expression for template-level separators at depth 0.
/// Handles nested brackets, strings, template literals, and comments.
fn find_expression_end(s: &str, extra_stops: &[u8]) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut depth: i32 = 0;

    while i < bytes.len() {
        let b = bytes[i];

        if depth == 0 && extra_stops.contains(&b) {
            return Some(i);
        }

        match b {
            b'{' => depth += 1,
            b'}' => {
                if depth == 0 {
                    return Some(i);
                }
                depth -= 1;
            }
            b'\'' | b'"' => {
                i = skip_string(bytes, i)?;
            }
            b'`' => {
                i = skip_template_literal(bytes, i)?;
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                i += 2;
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                if i + 1 < bytes.len() {
                    i += 1;
                }
            }
            b'(' => depth += 1,
            b')' => depth -= 1,
            b'[' => depth += 1,
            b']' => depth -= 1,
            _ => {}
        }
        i += 1;
    }
    None
}

/// Find end of each-block expression, stopping at ` as ` keyword at depth 0 or `}`.
/// Used as fallback when OXC can't parse the expression directly (JS mode with `as`).
fn find_each_expression_end(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut depth: i32 = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                if depth == 0 {
                    return Some(i);
                }
                depth -= 1;
            }
            b'\'' | b'"' => {
                i = skip_string(bytes, i)?;
            }
            b'`' => {
                i = skip_template_literal(bytes, i)?;
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                i += 2;
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                if i + 1 < bytes.len() {
                    i += 1;
                }
            }
            b'(' => depth += 1,
            b')' => depth -= 1,
            b'[' => depth += 1,
            b']' => depth -= 1,
            _ if depth == 0 => {
                if i > 0
                    && bytes[i - 1].is_ascii_whitespace()
                    && s[i..].starts_with("as")
                    && i + 2 < bytes.len()
                    && !bytes[i + 2].is_ascii_alphanumeric()
                    && bytes[i + 2] != b'_'
                {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

fn skip_string(bytes: &[u8], start: usize) -> Option<usize> {
    let quote = bytes[start];
    let mut i = start + 1;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        if bytes[i] == quote {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn skip_template_literal(bytes: &[u8], start: usize) -> Option<usize> {
    let mut i = start + 1;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        if bytes[i] == b'`' {
            return Some(i);
        }
        if bytes[i] == b'$' && i + 1 < bytes.len() && bytes[i + 1] == b'{' {
            i += 2;
            let mut depth = 1;
            while i < bytes.len() && depth > 0 {
                match bytes[i] {
                    b'{' => depth += 1,
                    b'}' => depth -= 1,
                    b'\'' | b'"' => {
                        i = skip_string(bytes, i)?;
                    }
                    b'`' => {
                        i = skip_template_literal(bytes, i)?;
                    }
                    b'\\' => {
                        i += 1;
                    }
                    _ => {}
                }
                i += 1;
            }
            continue;
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- find_expression_end tests (used by read_expression_until) --

    #[test]
    fn test_stop_at_equals() {
        assert_eq!(find_expression_end("x = 42}", &[b'=']), Some(2));
    }

    #[test]
    fn test_stop_at_equals_nested() {
        assert_eq!(find_expression_end("{ a = 1 } = obj}", &[b'=']), Some(10));
    }

    #[test]
    fn test_stop_at_comma() {
        assert_eq!(find_expression_end("a, b)", &[b',', b')']), Some(1));
    }

    #[test]
    fn test_stop_at_paren() {
        assert_eq!(find_expression_end("x)", &[b')']), Some(1));
    }

    // -- find_each_expression_end tests --

    #[test]
    fn test_each_expression_end() {
        assert_eq!(find_each_expression_end("items as item}"), Some(6));
    }

    #[test]
    fn test_each_expression_end_no_as() {
        assert_eq!(find_each_expression_end("items}"), Some(5));
    }

    #[test]
    fn test_each_expression_end_complex() {
        assert_eq!(
            find_each_expression_end("items.filter(x => x.ok) as item}"),
            Some(24)
        );
    }

    // -- OXC grammar-based boundary detection tests --

    #[test]
    fn test_oxc_grammar_boundary() {
        use oxc_allocator::Allocator;
        use oxc_parser::Parser as OxcParser;

        let alloc = Allocator::default();

        // OXC stops at expression boundary — grammar-based, not byte scanning
        let r = OxcParser::new(&alloc, "x + 1} rest", SourceType::mjs()).parse_expression();
        assert!(r.is_ok());
        assert_eq!(r.unwrap().span().end, 5);

        let r = OxcParser::new(&alloc, "foo(1, 2)} more", SourceType::mjs()).parse_expression();
        assert!(r.is_ok());
        assert_eq!(r.unwrap().span().end, 9);

        let r = OxcParser::new(&alloc, "{ a: 1 }} rest", SourceType::mjs()).parse_expression();
        assert!(r.is_ok());
        assert_eq!(r.unwrap().span().end, 8);

        // Regex with } in char class — grammar handles correctly
        let r = OxcParser::new(&alloc, "/[}]/.test(x)} rest", SourceType::mjs()).parse_expression();
        assert!(r.is_ok());
        assert_eq!(r.unwrap().span().end, 13);

        // TS as expression consumed by OXC
        let r = OxcParser::new(&alloc, "items as item}", SourceType::ts()).parse_expression();
        assert!(r.is_ok());
        assert_eq!(r.unwrap().span().end, 13);

        // JS mode with `as` — OXC fails (needs fallback)
        let r = OxcParser::new(&alloc, "items as item}", SourceType::mjs()).parse_expression();
        assert!(r.is_err());
    }
}
