use svelte_ast::node::AttributeNode;
use svelte_ast::root::{Script, ScriptContext};
use svelte_ast::span::Span;
use svelte_ast::attributes::{AttributeSequenceValue, AttributeValue};
use winnow::Result as ParseResult;
use winnow::combinator::peek;
use winnow::prelude::*;
use winnow::stream::Location;
use winnow::token::{any, literal, take_while};

use super::ParserInput;
use super::attribute::attribute_parser;
use super::oxc_parse::parse_program;

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

    // Parse content with OXC
    let ts = parser_input.state.ts;
    let (content, content_comments) = parse_program(&content_text, ts, content_offset)?;

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
            if a.name == "module" {
                return ScriptContext::Module;
            }
            if a.name == "context" {
                if let AttributeValue::Sequence(seq) = &a.value {
                    for item in seq {
                        if let AttributeSequenceValue::Text(t) = item {
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
