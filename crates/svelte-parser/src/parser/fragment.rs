use svelte_ast::node::FragmentNode;
use winnow::combinator::{alt, not, peek, repeat};
use winnow::prelude::*;
use winnow::token::{any, literal};
use winnow::Result;

use super::ParserInput;
use super::comment::comment_parser;
use super::element::element_parser;
use super::tag::expression_tag_parser;
use super::text::text_parser;

pub fn fragment_parser(parser_input: &mut ParserInput) -> Result<Vec<FragmentNode>> {
    repeat(0.., fragment_node_parser).parse_next(parser_input)
}

pub(crate) fn fragment_node_parser(parser_input: &mut ParserInput) -> Result<FragmentNode> {
    // Fail on closing tags - they are terminators for repeat_till in element_parser
    not(peek(literal("</"))).parse_next(parser_input)?;

    let c = peek(any).parse_next(parser_input)?;
    match c {
        '<' => alt((comment_parser, element_parser)).parse_next(parser_input),
        '{' => expression_tag_parser(parser_input),
        _ => text_parser(parser_input),
    }
}
