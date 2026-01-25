use std::cell::Cell;

use oxc_ast::ast::{Expression, FormalParameter, FormalParameterRest};
use oxc_span::{GetSpan, SourceType};

use crate::error::ErrorKind;
use crate::parser::read::context::read_pattern;
use crate::parser::read::expression::{loose_identifier, read_expression};
use crate::parser::span_offset::{
    shift_expression_spans, shift_formal_parameter_rest_spans, shift_formal_parameter_spans,
};
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

        let mut expression = read_each_expression(parser)?;

        parser.allow_whitespace();

        // Read context pattern (after "as")
        let context = if parser.eat("as") {
            parser.require_whitespace()?;
            read_pattern(parser)
        } else {
            // If no "as" and expression is SequenceExpression (e.g. `[10, 20], i`),
            // take only the first expression and reset index (matching reference behavior)
            if let Expression::SequenceExpression(seq) = &expression {
                let first_end = seq.expressions[0].span().end as usize;
                // Reconstruct: take the first expression
                if let Expression::SequenceExpression(seq) = expression {
                    let seq = seq.unbox();
                    expression = seq.expressions.into_iter().next().unwrap();
                }
                parser.index = first_end;
            }
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
                Some(name)
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

        parser.eat_required_with_loose("}", false)?;

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
        let expression = read_await_expression(parser)?;
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

        parser.eat_required_with_loose("}", false)?;

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
                type_params = Some(&parser.template[tp_start + 1..end - 1]);
                parser.index = end;
            }
        }

        parser.allow_whitespace();

        // Parse parameters: (param1, param2, ...)
        let params_start = parser.index;
        let (parameters, rest) = if parser.eat("(") {
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
            (Vec::new(), None)
        };

        parser.allow_whitespace();
        parser.eat_required("}")?;

        parser.stack.push(StackFrame::SnippetBlock {
            start,
            expression,
            parameters,
            rest,
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

    // Parse with OXC using parse_expression (no padding needed).
    let source_type = if parser.ts {
        SourceType::ts()
    } else {
        SourceType::mjs()
    };

    let snippet = parser.allocator.alloc_str(trimmed);
    let result = oxc_parser::Parser::new(parser.allocator, snippet, source_type).parse_expression();

    match result {
        Ok(mut e) => {
            shift_expression_spans(&mut e, start as u32);
            parser.index = effective_end;
            Ok(e)
        }
        Err(errors) => {
            if parser.loose {
                parser.index = effective_end;
                Ok(loose_identifier(parser, start, effective_end))
            } else {
                let msg = format!("JS parse error: {}", errors[0]);
                Err(parser.error(ErrorKind::JsParseError, start, msg))
            }
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

// ─── Await Expression Helpers ───────────────────────────────────────

/// Read the expression for an await block.
/// Scans for "then" or "catch" keywords at bracket depth 0 to find the expression boundary.
fn read_await_expression<'a>(parser: &mut Parser<'a>) -> Result<Expression<'a>, ParseError> {
    let start = parser.index;

    let keyword_pos = find_then_catch_keyword(parser.template, start);

    let expr_end = match keyword_pos {
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

    let source_type = if parser.ts {
        SourceType::ts()
    } else {
        SourceType::mjs()
    };

    let snippet = parser.allocator.alloc_str(trimmed);
    let result = oxc_parser::Parser::new(parser.allocator, snippet, source_type).parse_expression();

    match result {
        Ok(mut e) => {
            shift_expression_spans(&mut e, start as u32);
            parser.index = effective_end;
            Ok(e)
        }
        Err(errors) => {
            if parser.loose {
                parser.index = effective_end;
                Ok(loose_identifier(parser, start, effective_end))
            } else {
                let msg = format!("JS parse error: {}", errors[0]);
                Err(parser.error(ErrorKind::JsParseError, start, msg))
            }
        }
    }
}

/// Find "then" or "catch" keyword at bracket depth 0.
fn find_then_catch_keyword(template: &str, start: usize) -> Option<usize> {
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
                    // Check for "then" keyword
                    if ch == b't'
                        && i + 3 < bytes.len()
                        && bytes[i + 1] == b'h'
                        && bytes[i + 2] == b'e'
                        && bytes[i + 3] == b'n'
                        && i > start
                        && is_whitespace_byte(bytes[i - 1])
                        && (i + 4 >= bytes.len() || !is_identifier_byte(bytes[i + 4]))
                    {
                        return Some(i);
                    }
                    // Check for "catch" keyword
                    if ch == b'c'
                        && i + 4 < bytes.len()
                        && bytes[i + 1] == b'a'
                        && bytes[i + 2] == b't'
                        && bytes[i + 3] == b'c'
                        && bytes[i + 4] == b'h'
                        && i > start
                        && is_whitespace_byte(bytes[i - 1])
                        && (i + 5 >= bytes.len() || !is_identifier_byte(bytes[i + 5]))
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

// ─── Snippet Parameter Parsing ──────────────────────────────────────

/// Parse snippet parameters by wrapping in an arrow function and parsing with OXC.
/// Returns (items, rest) tuple.
fn parse_snippet_params<'a>(
    parser: &mut Parser<'a>,
    params_start: usize,
    params_source: &str,
) -> (
    Vec<FormalParameter<'a>>,
    Option<oxc_allocator::Box<'a, FormalParameterRest<'a>>>,
) {
    // Parse `<params> => {}` without padding, then shift spans.
    let snippet = format!("{} => {{}}", params_source);
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
                params_start,
                format!("Snippet params parse error: {}", first_err),
            );
        }
        return (Vec::new(), None);
    }

    let program = result.program;
    let body = program.body;
    if body.len() != 1 {
        return (Vec::new(), None);
    }

    let stmt = body.into_iter().next().unwrap();
    match stmt {
        oxc_ast::ast::Statement::ExpressionStatement(expr_stmt) => {
            let inner = expr_stmt.unbox();
            match inner.expression {
                Expression::ArrowFunctionExpression(arrow) => {
                    let arrow = arrow.unbox();
                    let offset = params_start as u32;
                    let formal_params = arrow.params.unbox();

                    let items: Vec<FormalParameter<'a>> = formal_params
                        .items
                        .into_iter()
                        .map(|mut param| {
                            shift_formal_parameter_spans(&mut param, offset);
                            param
                        })
                        .collect();

                    let rest = formal_params.rest.map(|mut r| {
                        shift_formal_parameter_rest_spans(&mut r, offset);
                        r
                    });

                    (items, rest)
                }
                _ => (Vec::new(), None),
            }
        }
        _ => (Vec::new(), None),
    }
}
