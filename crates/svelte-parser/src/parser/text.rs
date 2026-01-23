use svelte_ast::node::FragmentNode;
use svelte_ast::span::Span;
use svelte_ast::text::Text;
use winnow::Result as ParseResult;
use winnow::prelude::*;
use winnow::stream::Location;
use winnow::token::take_while;

use super::ParserInput;

pub fn text_parser(parser_input: &mut ParserInput) -> ParseResult<FragmentNode> {
    let start = parser_input.input.current_token_start();

    let data = take_while(1.., |c: char| !matches!(c, '<' | '{')).parse_next(parser_input)?;

    let end = parser_input.input.previous_token_end();

    Ok(FragmentNode::Text(Text {
        span: Span::new(start, end),
        data: data.to_string(),
        raw: data.to_string(),
    }))
}
