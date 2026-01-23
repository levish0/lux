use svelte_ast::node::FragmentNode;
use svelte_ast::span::Span;
use svelte_ast::text::Comment;
use winnow::combinator::delimited;
use winnow::prelude::*;
use winnow::stream::Location;
use winnow::token::take_until;
use winnow::Result;

use super::ParserInput;

pub fn comment_parser(parser_input: &mut ParserInput) -> Result<FragmentNode> {
    let start = parser_input.current_token_start();

    let data: &str = delimited("<!--", take_until(0.., "-->"), "-->")
        .parse_next(parser_input)?;

    let end = parser_input.previous_token_end();

    Ok(FragmentNode::Comment(Comment {
        span: Span::new(start, end),
        data: data.to_string(),
    }))
}
