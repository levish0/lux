use oxc_ast::ast::{Expression, FormalParameter};
use oxc_span::SourceType;

use crate::error::ErrorKind;
use crate::parser::bracket::match_bracket;
use crate::parser::span_offset::shift_formal_parameter_spans;
use crate::parser::Parser;

/// Read a pattern (with optional type annotation) at the current parser position.
/// Returns FormalParameter which contains both the pattern and type annotation.
///
/// Reference: context.js - read_pattern + read_type_annotation
pub fn read_pattern<'a>(parser: &mut Parser<'a>) -> Option<FormalParameter<'a>> {
    let start = parser.index;

    // 1. Try identifier first
    let (name, id_start, id_end) = parser.read_identifier();
    if !name.is_empty() {
        // Parse identifier pattern with optional type annotation
        let pattern_string = &parser.template[id_start..id_end];
        let mut param = parse_as_formal_parameter(parser, pattern_string, id_start)?;

        // Check for type annotation using "_ as " trick
        if let Some(type_ann) = read_type_annotation(parser) {
            param.type_annotation = Some(type_ann);
        }

        return Some(param);
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

    parser.index = bracket_end;

    // Parse pattern (without type annotation first)
    let pattern_string = &parser.template[start..bracket_end];
    let mut param = parse_as_formal_parameter(parser, pattern_string, start)?;

    // Check for type annotation using "_ as " trick
    if let Some(type_ann) = read_type_annotation(parser) {
        param.type_annotation = Some(type_ann);
    }

    Some(param)
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

/// Read type annotation using the "_ as " trick.
/// Reference: context.js:76-116 - read_type_annotation
///
/// We trick OXC into parsing the type annotation by constructing:
/// `_ as <type>` and then extracting the typeAnnotation from TSAsExpression.
fn read_type_annotation<'a>(
    parser: &mut Parser<'a>,
) -> Option<oxc_allocator::Box<'a, oxc_ast::ast::TSTypeAnnotation<'a>>> {
    let start = parser.index;
    parser.allow_whitespace();

    if !parser.eat(":") {
        parser.index = start;
        return None;
    }

    // First find the end of the type annotation using heuristic
    let type_start = parser.index;
    let type_end = find_annotation_end(parser.template, type_start);

    if type_end <= type_start {
        parser.index = start;
        return None;
    }

    // Extract type string and build "_ as <type>" for OXC to parse
    let type_str = &parser.template[type_start..type_end];
    let trimmed = type_str.trim();

    if trimmed.is_empty() {
        parser.index = start;
        return None;
    }

    // Replace ?<whitespace>: with : to handle optional parameters
    let type_cleaned = replace_optional_colon(trimmed);

    let snippet = format!("_ as {}", type_cleaned);
    let snippet_str = parser.allocator.alloc_str(&snippet);

    let source_type = SourceType::ts();
    let result =
        oxc_parser::Parser::new(parser.allocator, snippet_str, source_type).parse_expression();

    let expression = match result {
        Ok(expr) => expr,
        Err(_) => {
            parser.index = start;
            return None;
        }
    };

    // Extract typeAnnotation from TSAsExpression
    match expression {
        Expression::TSAsExpression(ts_as) => {
            let ts_as = ts_as.unbox();
            parser.index = type_end;

            // Build TSTypeAnnotation with correct spans
            let type_annotation = oxc_ast::ast::TSTypeAnnotation {
                span: oxc_span::Span::new(start as u32, type_end as u32),
                type_annotation: fix_type_spans(ts_as.type_annotation, type_start as u32),
            };
            Some(oxc_allocator::Box::new_in(type_annotation, parser.allocator))
        }
        _ => {
            parser.index = start;
            None
        }
    }
}

/// Fix type annotation spans to match original template positions.
/// The "_ as " prefix is 5 characters, so we need to offset by (type_start - 5).
fn fix_type_spans(mut ts_type: oxc_ast::ast::TSType, type_start: u32) -> oxc_ast::ast::TSType {
    // The type was parsed from "_ as <type>" where <type> starts at offset 5
    // We need to shift spans by (type_start - 5) to get original positions
    let offset = type_start.saturating_sub(5);

    // For now, just update the outer span - inner spans are complex to update
    // This is a simplification; full implementation would recursively update all spans
    match &mut ts_type {
        oxc_ast::ast::TSType::TSNumberKeyword(t) => {
            t.span = oxc_span::Span::new(t.span.start + offset, t.span.end + offset);
        }
        oxc_ast::ast::TSType::TSStringKeyword(t) => {
            t.span = oxc_span::Span::new(t.span.start + offset, t.span.end + offset);
        }
        oxc_ast::ast::TSType::TSBooleanKeyword(t) => {
            t.span = oxc_span::Span::new(t.span.start + offset, t.span.end + offset);
        }
        oxc_ast::ast::TSType::TSTypeReference(t) => {
            t.span = oxc_span::Span::new(t.span.start + offset, t.span.end + offset);
        }
        oxc_ast::ast::TSType::TSArrayType(t) => {
            t.span = oxc_span::Span::new(t.span.start + offset, t.span.end + offset);
        }
        _ => {
            // For other types, leave spans as-is for now
        }
    }

    ts_type
}

/// Find where a type annotation ends. Stops at `=`, `,`, `)`, `}`, or end of template.
/// Handles nested generics like `Array<string>` or `Map<string, number>`.
fn find_annotation_end(template: &str, start: usize) -> usize {
    let bytes = template.as_bytes();
    let mut i = start;
    let mut angle_depth = 0; // for < > nesting
    let mut paren_depth = 0; // for ( ) nesting

    while i < bytes.len() {
        match bytes[i] {
            b'<' => angle_depth += 1,
            b'>' => {
                if angle_depth > 0 {
                    angle_depth -= 1;
                }
            }
            b'(' => paren_depth += 1,
            b')' => {
                if paren_depth > 0 {
                    paren_depth -= 1;
                } else if angle_depth == 0 {
                    return i;
                }
            }
            b'=' | b',' | b'}' if angle_depth == 0 && paren_depth == 0 => {
                return i;
            }
            _ => {}
        }
        i += 1;
    }
    i
}

/// Replace `?<whitespace>:` with `:` to handle optional parameters
fn replace_optional_colon(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '?' {
            // Check if followed by whitespace and colon
            let mut j = i + 1;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            if j < chars.len() && chars[j] == ':' {
                // Skip the '?' and whitespace, keep ':'
                i = j;
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }

    result
}
