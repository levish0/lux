use svelte_ast::node::FragmentNode;
use svelte_ast::span::Span;
use svelte_ast::tags::{AttachTag, ConstTag, DebugTag, ExpressionTag, HtmlTag, RenderTag};
use swc_ecma_ast as swc;
use winnow::Result as ParseResult;
use winnow::combinator::{opt, peek};
use winnow::prelude::*;
use winnow::stream::Location;
use winnow::token::{literal, take_while};

use super::ParserInput;
use super::bracket::read_until_close_brace;
use super::expression::read_expression;
use super::swc_parse::{parse_expression, parse_var_decl};

/// Parse `{expression}` tag.
pub fn expression_tag_parser(parser_input: &mut ParserInput) -> ParseResult<FragmentNode> {
    let start = parser_input.current_token_start();
    let expression = read_expression(parser_input)?;
    let end = parser_input.previous_token_end();

    Ok(FragmentNode::ExpressionTag(ExpressionTag {
        span: Span::new(start, end),
        expression,
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
        _ => Err(winnow::error::ContextError::new()),
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
            let name: &str = take_while(1.., |c: char| {
                c.is_ascii_alphanumeric() || c == '_' || c == '$'
            })
            .parse_next(parser_input)?;
            identifiers.push(swc::Ident::new(
                name.into(),
                swc_common::DUMMY_SP,
                Default::default(),
            ));
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
        identifiers,
    }))
}

/// Parse `{@const declaration}`
fn const_tag_parser(parser_input: &mut ParserInput, start: usize) -> ParseResult<FragmentNode> {
    take_while(1.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;

    let content = read_until_close_brace(parser_input)?;
    literal("}").parse_next(parser_input)?;
    let end = parser_input.previous_token_end();

    let declaration = parse_var_decl(content, parser_input.state.ts)?;

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
