use std::cell::Cell;

use oxc_ast::ast::Expression;
use oxc_span::SourceType;

use crate::error::ErrorKind;
use crate::parser::span_offset::shift_expression_spans;
use crate::parser::{ParseError, Parser};

/// Read a JS/TS expression at the current parser position.
/// Port of reference `read/expression.js`.
///
/// Uses bracket tracking to find the expression boundary,
/// then parses with OXC. Advances `parser.index` past the expression.
pub fn read_expression<'a>(parser: &mut Parser<'a>) -> Result<Expression<'a>, ParseError> {
    let start = parser.index;

    // Find where the expression ends by locating the closing bracket.
    let end = find_expression_end(parser);

    if end <= start {
        if parser.loose {
            return Ok(loose_identifier(parser, start, start));
        }
        return Err(parser.error(
            ErrorKind::ExpectedExpression,
            start,
            "Expected expression".to_string(),
        ));
    }

    let expr_source = &parser.template[start..end];

    // Trim trailing whitespace from expression
    let trimmed = expr_source.trim_end();
    let effective_end = start + trimmed.len();

    if trimmed.is_empty() {
        if parser.loose {
            parser.index = end;
            return Ok(loose_identifier(parser, start, end));
        }
        parser.index = end;
        return Err(parser.error(
            ErrorKind::ExpectedExpression,
            start,
            "Expected expression".to_string(),
        ));
    }

    // Parse with OXC using parse_expression (no padding needed).
    let source_type = if parser.ts {
        SourceType::ts()
    } else {
        SourceType::mjs()
    };

    let snippet = parser.allocator.alloc_str(trimmed);
    let result =
        oxc_parser::Parser::new(parser.allocator, snippet, source_type).parse_expression();

    match result {
        Ok(mut e) => {
            shift_expression_spans(&mut e, start as u32);
            parser.index = effective_end;
            Ok(e)
        }
        Err(_errors) => {
            if parser.loose {
                parser.index = end;
                Ok(loose_identifier(parser, start, end))
            } else {
                let msg = format!("JS parse error: {}", _errors[0]);
                parser.index = end;
                Err(parser.error(ErrorKind::JsParseError, start, msg))
            }
        }
    }
}

/// Find where the current expression ends.
/// Scans forward tracking brackets, stops at the first unmatched `}` or `)`.
fn find_expression_end(parser: &Parser) -> usize {
    let bytes = parser.template.as_bytes();
    let mut i = parser.index;
    let mut brace_depth: i32 = 0;
    let mut paren_depth: i32 = 0;
    let mut bracket_depth: i32 = 0;

    while i < bytes.len() {
        let ch = bytes[i];
        match ch {
            b'{' => brace_depth += 1,
            b'}' => {
                if brace_depth == 0 {
                    return i;
                }
                brace_depth -= 1;
            }
            b'(' => paren_depth += 1,
            b')' => {
                if paren_depth == 0 {
                    return i;
                }
                paren_depth -= 1;
            }
            b'[' => bracket_depth += 1,
            b']' => {
                if bracket_depth == 0 {
                    return i;
                }
                bracket_depth -= 1;
            }
            b'\'' | b'"' | b'`' => {
                i = skip_string_in_expr(bytes, i);
                continue;
            }
            b'/' => {
                if i + 1 < bytes.len() {
                    let next = bytes[i + 1];
                    if next == b'/' {
                        // Line comment
                        while i < bytes.len() && bytes[i] != b'\n' {
                            i += 1;
                        }
                        continue;
                    } else if next == b'*' {
                        // Block comment
                        i += 2;
                        while i + 1 < bytes.len() {
                            if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                                i += 2;
                                break;
                            }
                            i += 1;
                        }
                        continue;
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }

    parser.template.len()
}

/// Skip a string literal during expression boundary scanning.
fn skip_string_in_expr(bytes: &[u8], start: usize) -> usize {
    let quote = bytes[start];
    let mut i = start + 1;
    while i < bytes.len() {
        let ch = bytes[i];
        if ch == b'\\' {
            i += 2;
            continue;
        }
        if ch == quote {
            return i + 1;
        }
        if quote == b'`' && ch == b'$' && i + 1 < bytes.len() && bytes[i + 1] == b'{' {
            // Template literal interpolation
            i += 2;
            let mut depth = 1u32;
            while i < bytes.len() && depth > 0 {
                match bytes[i] {
                    b'{' => depth += 1,
                    b'}' => depth -= 1,
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
    i
}

/// Create a dummy Identifier expression for loose/error recovery.
pub fn loose_identifier<'a>(parser: &Parser<'a>, start: usize, end: usize) -> Expression<'a> {
    Expression::Identifier(oxc_allocator::Box::new_in(
        oxc_ast::ast::IdentifierReference {
            span: oxc_span::Span::new(start as u32, end as u32),
            name: oxc_span::Atom::from(""),
            reference_id: Cell::new(None),
        },
        parser.allocator,
    ))
}
