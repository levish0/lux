use std::cell::Cell;

use oxc_ast::ast::{BindingPattern, Expression};
use oxc_span::SourceType;

use crate::error::ErrorKind;
use crate::parser::read::context::read_pattern;
use crate::parser::read::expression::{extract_expression, loose_identifier, read_expression};
use crate::parser::{AwaitPhase, ParseError, Parser, StackFrame};

use super::{is_identifier_byte, is_whitespace_byte, skip_string_bytes, skip_to_closing_brace};

/// `{#...}` — open block (if, each, await, key, snippet)
pub fn open(parser: &mut Parser) -> Result<(), ParseError> {
    let mut start = parser.index - 2;
    while start > 0 && parser.template.as_bytes()[start] != b'{' {
        start -= 1;
    }

    if parser.eat("if") {
        parser.require_whitespace()?;

        let test = read_expression(parser)?;

        parser.allow_whitespace();
        parser.eat_required("}")?;

        parser.stack.push(StackFrame::IfBlock {
            start,
            elseif: false,
            test,
            consequent: None,
        });
        parser.fragments.push(Vec::new());

        return Ok(());
    }

    if parser.eat("each") {
        parser.require_whitespace()?;

        let expression = read_each_expression(parser)?;

        parser.allow_whitespace();

        // Read context pattern (after "as")
        let context = if parser.eat("as") {
            parser.require_whitespace()?;
            read_pattern(parser)
        } else {
            None
        };

        parser.allow_whitespace();

        // Read index variable (after ",")
        let index = if parser.eat(",") {
            parser.allow_whitespace();
            let (name, _start, _end) = parser.read_identifier();
            if name.is_empty() {
                if !parser.loose {
                    return Err(parser.error(
                        ErrorKind::ExpectedToken,
                        parser.index,
                        "Expected identifier".to_string(),
                    ));
                }
                None
            } else {
                parser.allow_whitespace();
                Some(name.to_string())
            }
        } else {
            None
        };

        // Read key expression (inside parentheses)
        let key = if parser.eat("(") {
            parser.allow_whitespace();
            let k = read_expression(parser)?;
            parser.allow_whitespace();
            parser.eat_required(")")?;
            parser.allow_whitespace();
            Some(k)
        } else {
            None
        };

        parser.eat_required("}")?;

        parser.stack.push(StackFrame::EachBlock {
            start,
            expression,
            context,
            index,
            key,
            body: None,
        });
        parser.fragments.push(Vec::new());

        return Ok(());
    }

    if parser.eat("await") {
        parser.require_whitespace()?;
        let expression = read_expression(parser)?;
        parser.allow_whitespace();

        let mut value = None;
        let mut error = None;
        let phase;

        if parser.eat("then") {
            if parser.peek_whitespace_then(b'}') {
                parser.allow_whitespace();
            } else {
                parser.require_whitespace()?;
                value = read_pattern(parser);
                parser.allow_whitespace();
            }
            phase = AwaitPhase::Then;
        } else if parser.eat("catch") {
            if parser.peek_whitespace_then(b'}') {
                parser.allow_whitespace();
            } else {
                parser.require_whitespace()?;
                error = read_pattern(parser);
                parser.allow_whitespace();
            }
            phase = AwaitPhase::Catch;
        } else {
            phase = AwaitPhase::Pending;
        }

        parser.eat_required("}")?;

        parser.stack.push(StackFrame::AwaitBlock {
            start,
            expression,
            value,
            error,
            pending: None,
            then: None,
            phase,
        });
        parser.fragments.push(Vec::new());

        return Ok(());
    }

    if parser.eat("key") {
        parser.require_whitespace()?;

        let expression = read_expression(parser)?;
        parser.allow_whitespace();

        parser.eat_required("}")?;

        parser
            .stack
            .push(StackFrame::KeyBlock { start, expression });
        parser.fragments.push(Vec::new());

        return Ok(());
    }

    if parser.eat("snippet") {
        parser.require_whitespace()?;

        let (name, name_start, name_end) = parser.read_identifier();
        if name.is_empty() && !parser.loose {
            return Err(parser.error(
                ErrorKind::ExpectedToken,
                parser.index,
                "Expected identifier".to_string(),
            ));
        }

        // Build identifier expression for the snippet name
        let expression = Expression::Identifier(oxc_allocator::Box::new_in(
            oxc_ast::ast::IdentifierReference {
                span: oxc_span::Span::new(name_start as u32, name_end as u32),
                name: oxc_span::Atom::from(name),
                reference_id: Cell::new(None),
            },
            parser.allocator,
        ));

        parser.allow_whitespace();

        // Handle optional type parameters: {#snippet foo<T>(...)}
        let mut type_params = None;
        if parser.ts && parser.match_str("<") {
            let tp_start = parser.index;
            let pointy_brackets: &[(char, char)] = &[('<', '>')];
            if let Some(end) =
                crate::parser::bracket::match_bracket(parser.template, tp_start, pointy_brackets)
            {
                type_params = Some(parser.template[tp_start + 1..end - 1].to_string());
                parser.index = end;
            }
        }

        parser.allow_whitespace();

        // Parse parameters: (param1, param2, ...)
        let params_start = parser.index;
        let parameters = if parser.eat("(") {
            // Find the matching )
            let mut paren_depth = 1u32;
            while parser.index < parser.template.len() {
                let ch = parser.template.as_bytes()[parser.index];
                match ch {
                    b'(' => paren_depth += 1,
                    b')' => {
                        paren_depth -= 1;
                        if paren_depth == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
                parser.index += 1;
            }
            parser.eat_required(")")?;
            let params_end = parser.index;

            let params_source = &parser.template[params_start..params_end];
            parse_snippet_params(parser, params_start, params_source)
        } else {
            Vec::new()
        };

        parser.allow_whitespace();
        parser.eat_required("}")?;

        parser.stack.push(StackFrame::SnippetBlock {
            start,
            expression,
            parameters,
            type_params,
        });
        parser.fragments.push(Vec::new());

        return Ok(());
    }

    if !parser.loose {
        return Err(parser.error(
            ErrorKind::ExpectedToken,
            parser.index,
            "Expected block type (if, each, await, key, or snippet)".to_string(),
        ));
    }
    skip_to_closing_brace(parser);
    Ok(())
}

// ─── Each Expression Helpers ────────────────────────────────────────

/// Read the collection expression for an each block.
/// Scans for the "as" keyword at bracket depth 0 to find the expression boundary.
fn read_each_expression<'a>(parser: &mut Parser<'a>) -> Result<Expression<'a>, ParseError> {
    let start = parser.index;

    let as_pos = find_as_keyword(parser.template, start);

    let expr_end = match as_pos {
        Some(pos) => {
            let slice = &parser.template[start..pos];
            start + slice.trim_end().len()
        }
        None => find_closing_brace_pos(parser.template, start),
    };

    if expr_end <= start {
        if parser.loose {
            parser.index = expr_end;
            return Ok(loose_identifier(parser, start, start));
        }
        return Err(parser.error(
            ErrorKind::ExpectedExpression,
            start,
            "Expected expression".to_string(),
        ));
    }

    let expr_source = &parser.template[start..expr_end];
    let trimmed = expr_source.trim_end();
    let effective_end = start + trimmed.len();

    if trimmed.is_empty() {
        parser.index = expr_end;
        return Ok(loose_identifier(parser, start, expr_end));
    }

    // Parse with OXC
    let prefix: String = parser.template[..start]
        .chars()
        .map(|c| if c == '\n' { '\n' } else { ' ' })
        .collect();
    let padded = format!("{}{};", prefix, trimmed);
    let padded_str = parser.allocator.alloc_str(&padded);

    let source_type = if parser.ts {
        SourceType::ts()
    } else {
        SourceType::mjs()
    };

    let result = oxc_parser::Parser::new(parser.allocator, padded_str, source_type).parse();

    if !result.errors.is_empty() {
        if parser.loose {
            parser.index = effective_end;
            return Ok(loose_identifier(parser, start, effective_end));
        }
        let first_err = &result.errors[0];
        return Err(parser.error(
            ErrorKind::JsParseError,
            start,
            format!("JS parse error: {}", first_err),
        ));
    }

    let program = result.program;
    let expr = extract_expression(program);

    match expr {
        Some(e) => {
            parser.index = effective_end;
            Ok(e)
        }
        None => {
            parser.index = effective_end;
            Ok(loose_identifier(parser, start, effective_end))
        }
    }
}

/// Find the "as" keyword at bracket depth 0.
fn find_as_keyword(template: &str, start: usize) -> Option<usize> {
    let bytes = template.as_bytes();
    let mut i = start;
    let mut brace_depth: i32 = 0;
    let mut paren_depth: i32 = 0;
    let mut bracket_depth: i32 = 0;

    while i < bytes.len() {
        let ch = bytes[i];
        match ch {
            b'{' => brace_depth += 1,
            b'}' => {
                if brace_depth == 0 {
                    return None;
                }
                brace_depth -= 1;
            }
            b'(' => paren_depth += 1,
            b')' => paren_depth -= 1,
            b'[' => bracket_depth += 1,
            b']' => bracket_depth -= 1,
            b'\'' | b'"' | b'`' => {
                i = skip_string_bytes(bytes, i);
                continue;
            }
            _ => {
                if brace_depth == 0 && paren_depth == 0 && bracket_depth == 0 {
                    if ch == b'a'
                        && i + 1 < bytes.len()
                        && bytes[i + 1] == b's'
                        && i > start
                        && is_whitespace_byte(bytes[i - 1])
                        && (i + 2 >= bytes.len() || !is_identifier_byte(bytes[i + 2]))
                    {
                        return Some(i);
                    }
                }
            }
        }
        i += 1;
    }
    None
}

/// Find the position of the closing `}` at bracket depth 0.
fn find_closing_brace_pos(template: &str, start: usize) -> usize {
    let bytes = template.as_bytes();
    let mut i = start;
    let mut brace_depth: i32 = 0;

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
            b'\'' | b'"' | b'`' => {
                i = skip_string_bytes(bytes, i);
                continue;
            }
            _ => {}
        }
        i += 1;
    }
    template.len()
}

// ─── Snippet Parameter Parsing ──────────────────────────────────────

/// Parse snippet parameters by wrapping in an arrow function and parsing with OXC.
fn parse_snippet_params<'a>(
    parser: &mut Parser<'a>,
    params_start: usize,
    params_source: &str,
) -> Vec<BindingPattern<'a>> {
    let prefix: String = parser.template[..params_start]
        .chars()
        .map(|c| if c == '\n' { '\n' } else { ' ' })
        .collect();
    let padded = format!("{}{} => {{}}", prefix, params_source);
    let padded_str = parser.allocator.alloc_str(&padded);

    let source_type = if parser.ts {
        SourceType::ts()
    } else {
        SourceType::mjs()
    };

    let result = oxc_parser::Parser::new(parser.allocator, padded_str, source_type).parse();

    if !result.errors.is_empty() {
        if !parser.loose {
            let first_err = &result.errors[0];
            parser.error(
                ErrorKind::JsParseError,
                params_start,
                format!("Snippet params parse error: {}", first_err),
            );
        }
        return Vec::new();
    }

    let program = result.program;
    let body = program.body;
    if body.len() != 1 {
        return Vec::new();
    }

    let stmt = body.into_iter().next().unwrap();
    match stmt {
        oxc_ast::ast::Statement::ExpressionStatement(expr_stmt) => {
            let inner = expr_stmt.unbox();
            match inner.expression {
                Expression::ArrowFunctionExpression(arrow) => {
                    let arrow = arrow.unbox();
                    arrow
                        .params
                        .unbox()
                        .items
                        .into_iter()
                        .map(|param| param.pattern)
                        .collect()
                }
                _ => Vec::new(),
            }
        }
        _ => Vec::new(),
    }
}
