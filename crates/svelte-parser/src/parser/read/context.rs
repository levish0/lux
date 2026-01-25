use oxc_ast::ast::{Expression, FormalParameter};
use oxc_span::SourceType;

use crate::error::ErrorKind;
use crate::parser::Parser;
use crate::parser::bracket::match_bracket;
use crate::parser::span_offset::shift_formal_parameter_spans;

/// Read a pattern (with optional type annotation) at the current parser position.
/// Returns FormalParameter which contains both the pattern and type annotation.
///
/// Parses by wrapping as `(pattern) => {}` and extracting the FormalParameter.
pub fn read_pattern<'a>(parser: &mut Parser<'a>) -> Option<FormalParameter<'a>> {
    let start = parser.index;

    // 1. Try identifier first
    let (name, id_start, _id_end) = parser.read_identifier();
    if !name.is_empty() {
        // Check for type annotation (TS): identifier followed by `:`
        parser.allow_whitespace();
        let pattern_end = if parser.match_str(":") {
            // Has type annotation - find the end
            find_annotation_end(parser.template, parser.index)
        } else {
            parser.index
        };
        parser.index = pattern_end;

        // Parse with OXC: `(pattern) => {}`
        let pattern_string = &parser.template[id_start..pattern_end];
        return parse_as_formal_parameter(parser, pattern_string, id_start);
    }

    // 2. Check for destructuring pattern
    let ch = parser.template.as_bytes().get(parser.index).copied();
    if ch != Some(b'{') && ch != Some(b'[') {
        if !parser.loose {
            parser.error(
                ErrorKind::ExpectedExpression,
                start,
                "Expected pattern".to_string(),
            );
        }
        return None;
    }

    // 3. Use match_bracket to find the end of the pattern
    let default_brackets: &[(char, char)] = &[('{', '}'), ('(', ')'), ('[', ']')];
    let bracket_end = match match_bracket(parser.template, start, default_brackets) {
        Some(end) => end,
        None => {
            if !parser.loose {
                parser.error(
                    ErrorKind::ExpectedToken,
                    start,
                    "Unterminated pattern".to_string(),
                );
            }
            return None;
        }
    };

    // Check for type annotation after bracket
    parser.index = bracket_end;
    parser.allow_whitespace();
    let pattern_end = if parser.match_str(":") {
        find_annotation_end(parser.template, parser.index)
    } else {
        bracket_end
    };
    parser.index = pattern_end;

    let pattern_string = &parser.template[start..pattern_end];
    parse_as_formal_parameter(parser, pattern_string, start)
}

/// Parse a pattern string as `(pattern) => {}` and extract the FormalParameter.
fn parse_as_formal_parameter<'a>(
    parser: &mut Parser<'a>,
    pattern_string: &str,
    original_start: usize,
) -> Option<FormalParameter<'a>> {
    // Wrap as arrow function: `(pattern) => {}`
    let paren_prefix_len = 1u32; // "("
    let snippet = format!("({}) => {{}}", pattern_string);
    let snippet_str = parser.allocator.alloc_str(&snippet);

    let source_type = if parser.ts {
        SourceType::ts()
    } else {
        SourceType::mjs()
    };

    let result = oxc_parser::Parser::new(parser.allocator, snippet_str, source_type).parse();

    if !result.errors.is_empty() {
        if !parser.loose {
            let first_err = &result.errors[0];
            parser.error(
                ErrorKind::JsParseError,
                original_start,
                format!("Pattern parse error: {}", first_err),
            );
        }
        return None;
    }

    // Extract FormalParameter from arrow function
    let program = result.program;
    let mut param = extract_formal_parameter(program)?;

    // Shift spans: pattern starts at byte 1 ("(") in snippet, but at `original_start` in original
    let offset = original_start as u32 - paren_prefix_len;
    shift_formal_parameter_spans(&mut param, offset);

    Some(param)
}

/// Extract the first FormalParameter from a parsed `(pattern) => {}`
fn extract_formal_parameter(program: oxc_ast::ast::Program) -> Option<FormalParameter> {
    let body = program.body;
    if body.len() != 1 {
        return None;
    }

    let stmt = body.into_iter().next()?;
    match stmt {
        oxc_ast::ast::Statement::ExpressionStatement(expr_stmt) => {
            let inner = expr_stmt.unbox();
            match inner.expression {
                Expression::ArrowFunctionExpression(arrow) => {
                    let arrow = arrow.unbox();
                    let params = arrow.params.unbox();
                    if params.items.is_empty() {
                        return None;
                    }
                    params.items.into_iter().next()
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// Find where a type annotation ends. Stops at `=`, `,`, `)`, `}`, or end of template.
/// Handles nested generics like `Array<string>` or `Map<string, number>`.
fn find_annotation_end(template: &str, start: usize) -> usize {
    let bytes = template.as_bytes();
    let mut i = start;
    let mut depth = 0; // for < > nesting

    while i < bytes.len() {
        match bytes[i] {
            b'<' => depth += 1,
            b'>' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            b'=' | b',' | b')' | b'}' if depth == 0 => {
                return i;
            }
            _ => {}
        }
        i += 1;
    }
    i
}
