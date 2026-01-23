use svelte_ast::node::FragmentNode;
use winnow::combinator::{alt, not, peek, repeat};
use winnow::prelude::*;
use winnow::token::{any, literal, take};
use winnow::Result;

use super::ParserInput;
use super::block::block_parser;
use super::comment::comment_parser;
use super::element::element_parser;
use super::tag::expression_tag_parser;
use super::text::text_parser;

pub fn fragment_parser(parser_input: &mut ParserInput) -> Result<Vec<FragmentNode>> {
    repeat(0.., fragment_node_parser).parse_next(parser_input)
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
            // Peek 2 chars to distinguish {# (block) from {expression}
            let two: Result<&str> = peek(take(2usize)).parse_next(parser_input);
            match two {
                Ok("{#") => block_parser(parser_input),
                _ => expression_tag_parser(parser_input),
            }
        }
        _ => text_parser(parser_input),
    }
}
