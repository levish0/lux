use svelte_ast::node::FragmentNode;
use svelte_ast::root::ScriptContext;
use winnow::combinator::{alt, not, peek};
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
        // Skip whitespace-only lookahead check — just try to parse
        if parser_input.input.is_empty() {
            break;
        }

        // Check for <script> at the document level
        let c = peek(any).parse_next(parser_input)?;
        if c == '<' {
            // Peek ahead: "<script" or "<style"
            let after_lt: Result<&str> = peek(take(7usize)).parse_next(parser_input);
            if let Ok(s) = after_lt {
                if s == "<script" {
                    // Check that the char after "<script" is whitespace or >
                    let full: Result<&str> = peek(take(8usize)).parse_next(parser_input);
                    let is_script = match full {
                        Ok(f) => {
                            let next_ch = f.chars().last().unwrap();
                            next_ch == '>' || next_ch.is_ascii_whitespace()
                        }
                        Err(_) => false,
                    };
                    if is_script {
                        let script = script_parser(parser_input)?;
                        match script.context {
                            ScriptContext::Module => {
                                parser_input.state.module = Some(script);
                            }
                            ScriptContext::Default => {
                                parser_input.state.instance = Some(script);
                            }
                        }
                        continue;
                    }
                }
                if s.starts_with("<style") {
                    // Check that the char after "<style" is whitespace or >
                    let full: Result<&str> = peek(take(7usize)).parse_next(parser_input);
                    let is_style = match full {
                        Ok(f) => {
                            let next_ch = f.chars().last().unwrap();
                            next_ch == '>' || next_ch.is_ascii_whitespace()
                        }
                        Err(_) => {
                            // Exactly "<style" with nothing after - treat as tag
                            false
                        }
                    };
                    if is_style {
                        let stylesheet = style_parser(parser_input)?;
                        parser_input.state.css = Some(stylesheet);
                        continue;
                    }
                }
            }
        }

        // Parse a regular fragment node
        match fragment_node_parser(parser_input) {
            Ok(node) => nodes.push(node),
            Err(_) => break,
        }
    }
    Ok(nodes)
}

pub(crate) fn fragment_node_parser(parser_input: &mut ParserInput) -> Result<FragmentNode> {
    // Fail on terminators: closing tags, block continuations, block closings
    not(peek(literal("</"))).parse_next(parser_input)?;
    not(peek(literal("{:"))).parse_next(parser_input)?;
    not(peek(literal("{/"))).parse_next(parser_input)?;

    let c = peek(any).parse_next(parser_input)?;
    match c {
        '<' => alt((comment_parser, element_parser)).parse_next(parser_input),
        '{' => {
            // Peek 2 chars to distinguish {# (block), {@ (special), {expression}
            let two: Result<&str> = peek(take(2usize)).parse_next(parser_input);
            match two {
                Ok("{#") => block_parser(parser_input),
                Ok("{@") => special_tag_parser(parser_input),
                _ => expression_tag_parser(parser_input),
            }
        }
        _ => text_parser(parser_input),
    }
}
