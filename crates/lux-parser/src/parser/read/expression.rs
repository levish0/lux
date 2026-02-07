use oxc_ast::ast::Expression;
use oxc_parser::Parser as OxcParser;
use oxc_span::SourceType;
use winnow::Result;
use winnow::error::ContextError;
use winnow::prelude::*;

use crate::input::Input;

/// Read a JS/TS expression from the input using OXC.
///
/// Assumes the caller has already consumed the opening delimiter (e.g., `{`).
/// Reads characters until the matching closing `}` is found,
/// then parses the extracted substring with OXC.
pub fn read_expression<'a>(input: &mut Input<'a>) -> Result<Expression<'a>> {
    let allocator = input.state.allocator;
    let ts = input.state.ts;

    let remaining: &str = &(*input.input);
    let end_offset = find_expression_end(remaining)
        .ok_or_else(ContextError::new)?;

    let expr_str = &remaining[..end_offset];

    if expr_str.trim().is_empty() {
        return Err(ContextError::new());
    }

    let source_type = if ts {
        SourceType::ts()
    } else {
        SourceType::mjs()
    };
    let result = OxcParser::new(allocator, expr_str, source_type).parse_expression();

    match result {
        Ok(expr) => {
            // Advance input past the expression
            let _ = winnow::token::take(end_offset).parse_next(input)?;
            Ok(expr)
        }
        Err(_diagnostics) => Err(ContextError::new()),
    }
}

/// Find the end of an expression by scanning for an unmatched `}`.
/// Handles nested brackets, strings, template literals, and comments.
fn find_expression_end(s: &str) -> Option<usize> {
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

    #[test]
    fn test_find_expression_end_simple() {
        assert_eq!(find_expression_end("name}"), Some(4));
    }

    #[test]
    fn test_find_expression_end_with_string() {
        assert_eq!(find_expression_end("'hello}'}"), Some(8));
    }

    #[test]
    fn test_find_expression_end_with_nested_braces() {
        assert_eq!(find_expression_end("obj.fn({a: 1})}"), Some(14));
    }

    #[test]
    fn test_find_expression_end_template_literal() {
        assert_eq!(find_expression_end("`${x}`}"), Some(6));
    }

    #[test]
    fn test_find_expression_end_parens() {
        assert_eq!(find_expression_end("(a + b)}"), Some(7));
    }
}
