use svelte_ast::node::FragmentNode;
use svelte_ast::root::ScriptContext;
use winnow::combinator::{alt, dispatch, not, opt, peek};
use winnow::prelude::*;
use winnow::token::{any, literal, take};
use winnow::Result;

use super::ParserInput;
use super::block::block_parser;
use super::comment::comment_parser;
use super::element::element_parser;
use super::css::style_parser;
use super::script::script_parser;
use super::tag::{expression_tag_parser, special_tag_parser};
use super::text::text_parser;

/// Parse the top-level document fragment, handling `<script>` and `<style>` specially.
pub fn document_parser(parser_input: &mut ParserInput) -> Result<Vec<FragmentNode>> {
    let mut nodes = Vec::new();
    loop {
        if parser_input.input.is_empty() {
            break;
        }

        // Try to parse <script> or <style> at document level
        if try_parse_script_or_style(parser_input)? {
            continue;
        }

        // Parse a regular fragment node
        match fragment_node_parser(parser_input) {
            Ok(node) => nodes.push(node),
            Err(_) => break,
        }
    }
    Ok(nodes)
}

/// Try to parse a top-level <script> or <style> tag.
/// Returns true if one was consumed.
fn try_parse_script_or_style(parser_input: &mut ParserInput) -> Result<bool> {
    // Only relevant if next char is '<'
    if opt(peek(literal("<s"))).parse_next(parser_input)?.is_none() {
        return Ok(false);
    }

    // Check for <script followed by whitespace or >
    if is_tag_start(parser_input, "<script") {
        let script = script_parser(parser_input)?;
        match script.context {
            ScriptContext::Module => {
                parser_input.state.module = Some(script);
            }
            ScriptContext::Default => {
                parser_input.state.instance = Some(script);
            }
        }
        return Ok(true);
    }

    // Check for <style followed by whitespace or >
    if is_tag_start(parser_input, "<style") {
        let stylesheet = style_parser(parser_input)?;
        parser_input.state.css = Some(stylesheet);
        return Ok(true);
    }

    Ok(false)
}

/// Check if the input starts with `tag_name` followed by whitespace or `>`.
fn is_tag_start(parser_input: &mut ParserInput, tag_name: &str) -> bool {
    let remaining: &str = &parser_input.input;
    if !remaining.starts_with(tag_name) {
        return false;
    }
    // Check char after tag name
    remaining.as_bytes().get(tag_name.len()).map_or(false, |&ch| {
        ch == b'>' || ch.is_ascii_whitespace()
    })
}

pub(crate) fn fragment_node_parser(parser_input: &mut ParserInput) -> Result<FragmentNode> {
    // Fail on terminators: closing tags, block continuations, block closings
    not(peek(literal("</"))).parse_next(parser_input)?;
    not(peek(literal("{:"))).parse_next(parser_input)?;
    not(peek(literal("{/"))).parse_next(parser_input)?;

    dispatch! {peek(any);
        '<' => alt((comment_parser, element_parser)),
        '{' => dispatch! {peek(take(2usize));
            "{#" => block_parser,
            "{@" => special_tag_parser,
            _ => expression_tag_parser,
        },
        _ => text_parser,
    }
    .parse_next(parser_input)
}
