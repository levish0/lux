use svelte_ast::JsNode;
use svelte_ast::node::FragmentNode;
use svelte_ast::span::Span;
use svelte_ast::tags::{AttachTag, ConstTag, DebugTag, ExpressionTag, HtmlTag, RenderTag};
use winnow::Result as ParseResult;
use winnow::combinator::{opt, peek};
use winnow::prelude::*;
use winnow::stream::Location;
use winnow::token::{literal, take_while};

use super::ParserInput;
use super::bracket::read_until_close_brace;
use super::expression::read_expression;
use super::oxc_parse::{is_call_expression, parse_expression, parse_var_decl, var_decl_count};
use crate::error::{ErrorKind, ParseError};

/// Parse `{expression}` tag.
pub fn expression_tag_parser(parser_input: &mut ParserInput) -> ParseResult<FragmentNode> {
    let start = parser_input.current_token_start();
    let expression = read_expression(parser_input)?;
    let end = parser_input.previous_token_end();

    Ok(FragmentNode::ExpressionTag(ExpressionTag {
        span: Span::new(start, end),
        expression,
        force_expression_loc: false,
    }))
}

/// Parse `{@keyword ...}` special tags.
/// Called after `{@` has been peeked but not consumed.
pub fn special_tag_parser(parser_input: &mut ParserInput) -> ParseResult<FragmentNode> {
    let start = parser_input.current_token_start();

    // Consume {@
    literal("{@").parse_next(parser_input)?;

    // Read keyword
    let keyword: &str =
        take_while(1.., |c: char| c.is_ascii_alphabetic()).parse_next(parser_input)?;

    match keyword {
        "html" => html_tag_parser(parser_input, start),
        "debug" => debug_tag_parser(parser_input, start),
        "const" => const_tag_parser(parser_input, start),
        "render" => render_tag_parser(parser_input, start),
        "attach" => attach_tag_parser(parser_input, start),
        _ => {
            let end = parser_input.current_token_start();
            parser_input.state.errors.push(ParseError::new(
                ErrorKind::ExpectedTag,
                Span::new(start, end),
                "Expected 'html', 'render', 'attach', 'const', or 'debug'",
            ));
            Err(winnow::error::ContextError::new())
        }
    }
}

/// Parse `{@html expression}`
fn html_tag_parser(parser_input: &mut ParserInput, start: usize) -> ParseResult<FragmentNode> {
    take_while(1.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;

    let offset = parser_input.current_token_start();
    let content = read_until_close_brace(parser_input)?;
    let expression = parse_expression(content, parser_input.state.ts, offset as u32)?;
    literal("}").parse_next(parser_input)?;
    let end = parser_input.previous_token_end();

    Ok(FragmentNode::HtmlTag(HtmlTag {
        span: Span::new(start, end),
        expression,
    }))
}

/// Parse `{@debug ident1, ident2, ...}`
fn debug_tag_parser(parser_input: &mut ParserInput, start: usize) -> ParseResult<FragmentNode> {
    take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;

    let mut identifiers = Vec::new();

    // Check if we immediately hit } (bare @debug with no identifiers)
    if opt(peek(literal("}"))).parse_next(parser_input)?.is_none() {
        loop {
            take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;
            let ident_start = parser_input.current_token_start();
            let name: &str = take_while(1.., |c: char| {
                c.is_ascii_alphanumeric() || c == '_' || c == '$'
            })
            .parse_next(parser_input)?;
            let ident_end = parser_input.current_token_start();
            identifiers.push(serde_json::json!({
                "type": "Identifier",
                "name": name,
                "start": ident_start,
                "end": ident_end
            }));
            take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;
            if opt(literal(",")).parse_next(parser_input)?.is_none() {
                break;
            }
        }
    }

    literal("}").parse_next(parser_input)?;
    let end = parser_input.previous_token_end();

    Ok(FragmentNode::DebugTag(DebugTag {
        span: Span::new(start, end),
        identifiers: JsNode(serde_json::Value::Array(identifiers)),
    }))
}

/// Parse `{@const declaration}`
fn const_tag_parser(parser_input: &mut ParserInput, start: usize) -> ParseResult<FragmentNode> {
    take_while(1.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;

    let offset = parser_input.current_token_start();
    let content = read_until_close_brace(parser_input)?;
    literal("}").parse_next(parser_input)?;
    let end = parser_input.previous_token_end();

    let declaration = parse_var_decl(content, parser_input.state.ts, offset as u32)?;

    // Validate: must have exactly one declarator
    if var_decl_count(&declaration) != 1 {
        parser_input.state.errors.push(ParseError::new(
            ErrorKind::ConstTagInvalidExpression,
            Span::new(start, end),
            "{@const ...} must consist of a single variable declaration",
        ));
        return Err(winnow::error::ContextError::new());
    }

    Ok(FragmentNode::ConstTag(ConstTag {
        span: Span::new(start, end),
        declaration,
    }))
}

/// Parse `{@render expression()}`
fn render_tag_parser(parser_input: &mut ParserInput, start: usize) -> ParseResult<FragmentNode> {
    take_while(1.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;

    let offset = parser_input.current_token_start();
    let content = read_until_close_brace(parser_input)?;
    let expression = parse_expression(content, parser_input.state.ts, offset as u32)?;

    // Validate: must be CallExpression or optional chain call
    if !is_call_expression(&expression) {
        parser_input.state.errors.push(ParseError::new(
            ErrorKind::RenderTagInvalidExpression,
            Span::new(offset, parser_input.current_token_start()),
            "`{@render ...}` tags can only contain call expressions",
        ));
        return Err(winnow::error::ContextError::new());
    }

    literal("}").parse_next(parser_input)?;
    let end = parser_input.previous_token_end();

    Ok(FragmentNode::RenderTag(RenderTag {
        span: Span::new(start, end),
        expression,
    }))
}

/// Parse `{@attach expression}`
fn attach_tag_parser(parser_input: &mut ParserInput, start: usize) -> ParseResult<FragmentNode> {
    take_while(1.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;

    let offset = parser_input.current_token_start();
    let content = read_until_close_brace(parser_input)?;
    let expression = parse_expression(content, parser_input.state.ts, offset as u32)?;
    literal("}").parse_next(parser_input)?;
    let end = parser_input.previous_token_end();

    Ok(FragmentNode::AttachTag(AttachTag {
        span: Span::new(start, end),
        expression,
    }))
}
