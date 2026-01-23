use svelte_ast::node::AttributeNode;
use svelte_ast::root::{Script, ScriptContext};
use svelte_ast::span::Span;
use svelte_ast::text::{JsComment, JsCommentKind};
use swc_common::BytePos;
use swc_common::comments::{Comment, CommentKind, SingleThreadedComments};
use swc_common::input::StringInput;
use swc_ecma_ast as swc;
use swc_ecma_parser::{EsSyntax, Syntax, TsSyntax};
use winnow::Result as ParseResult;
use winnow::combinator::peek;
use winnow::prelude::*;
use winnow::stream::Location;
use winnow::token::{any, literal, take_while};

use super::ParserInput;
use super::attribute::attribute_parser;

/// Parse a `<script>` tag and return a Script node.
/// Consumes from `<script` to `</script>`.
pub fn script_parser(parser_input: &mut ParserInput) -> ParseResult<Script> {
    let start = parser_input.current_token_start();

    // Consume <script
    literal("<script").parse_next(parser_input)?;

    // Parse attributes
    let attributes = parse_script_attributes(parser_input)?;

    // Consume >
    literal(">").parse_next(parser_input)?;

    // Determine context from attributes
    let context = detect_script_context(&attributes);

    // Read content until </script>
    let content_start = parser_input.current_token_start();
    let content_text = read_until_closing_script(parser_input)?;
    let content_end = content_start + content_text.len();
    let content_offset = content_start as u32;

    // Consume </script>
    literal("</script>").parse_next(parser_input)?;
    let end = parser_input.previous_token_end();

    // Parse content with SWC
    let ts = parser_input.state.ts;
    let (content, content_comments) = swc_parse_program(&content_text, ts, content_offset)?;

    Ok(Script {
        span: Span::new(start, end),
        context,
        content,
        content_comments,
        content_start,
        content_end,
        attributes,
    })
}

fn parse_script_attributes(parser_input: &mut ParserInput) -> ParseResult<Vec<AttributeNode>> {
    // Script tag attributes should not have expression interpolation in quoted values
    // e.g. generics="T extends { foo: number }" should be plain text
    parser_input.state.text_only_attributes = true;
    let mut attributes = Vec::new();
    loop {
        take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;
        let next = peek(any).parse_next(parser_input)?;
        if next == '>' || next == '/' {
            break;
        }
        attributes.push(attribute_parser(parser_input)?);
    }
    parser_input.state.text_only_attributes = false;
    Ok(attributes)
}

fn detect_script_context(attributes: &[AttributeNode]) -> ScriptContext {
    for attr in attributes {
        if let AttributeNode::Attribute(a) = attr {
            // <script module> or <script context="module">
            if a.name == "module" {
                return ScriptContext::Module;
            }
            if a.name == "context" {
                if let svelte_ast::attributes::AttributeValue::Sequence(seq) = &a.value {
                    for item in seq {
                        if let svelte_ast::attributes::AttributeSequenceValue::Text(t) = item {
                            if t.data == "module" {
                                return ScriptContext::Module;
                            }
                        }
                    }
                }
            }
        }
    }
    ScriptContext::Default
}

/// Read raw text content until `</script>` without consuming the closing tag.
fn read_until_closing_script(parser_input: &mut ParserInput) -> ParseResult<String> {
    let mut buf = String::new();
    loop {
        let check: ParseResult<&str> = peek(literal("</script>")).parse_next(parser_input);
        if check.is_ok() {
            break;
        }
        let c: char = any.parse_next(parser_input)?;
        buf.push(c);
    }
    Ok(buf)
}

fn swc_parse_program(
    source: &str,
    ts: bool,
    offset: u32,
) -> ParseResult<(swc::Program, Vec<JsComment>)> {
    let syntax = if ts {
        Syntax::Typescript(TsSyntax {
            tsx: true,
            ..Default::default()
        })
    } else {
        Syntax::Es(EsSyntax {
            jsx: true,
            ..Default::default()
        })
    };

    let comments = SingleThreadedComments::default();
    let input = StringInput::new(
        source,
        BytePos(offset),
        BytePos(offset + source.len() as u32),
    );
    let mut parser = swc_ecma_parser::Parser::new(syntax, input, Some(&comments));

    let module = parser.parse_module().map_err(|e| {
        e.into_diagnostic(&swc_common::errors::Handler::with_emitter(
            true,
            false,
            Box::new(swc_common::errors::EmitterWriter::new(
                Box::new(std::io::sink()),
                None,
                false,
                false,
            )),
        ))
        .cancel();
        winnow::error::ContextError::new()
    })?;

    // Collect all comments from SWC into JsComment format
    let js_comments = collect_comments(&comments);

    Ok((swc::Program::Module(module), js_comments))
}

fn collect_comments(comments: &SingleThreadedComments) -> Vec<JsComment> {
    let mut result = Vec::new();

    let (leading_map, trailing_map) = comments.borrow_all();

    for (_pos, comment_vec) in leading_map.iter() {
        for comment in comment_vec {
            result.push(swc_comment_to_js_comment(comment));
        }
    }
    for (_pos, comment_vec) in trailing_map.iter() {
        for comment in comment_vec {
            result.push(swc_comment_to_js_comment(comment));
        }
    }

    // Sort by start position
    result.sort_by_key(|c| c.span.start);
    result
}

fn swc_comment_to_js_comment(comment: &Comment) -> JsComment {
    let start = comment.span.lo.0 as usize;
    let end = comment.span.hi.0 as usize;
    let kind = match comment.kind {
        CommentKind::Line => JsCommentKind::Line,
        CommentKind::Block => JsCommentKind::Block,
    };
    JsComment {
        span: Span::new(start, end),
        kind,
        value: comment.text.to_string(),
    }
}
