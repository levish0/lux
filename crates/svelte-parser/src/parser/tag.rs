use svelte_ast::node::FragmentNode;
use svelte_ast::span::Span;
use svelte_ast::tags::ExpressionTag;
use winnow::stream::Location;
use winnow::Result;

use super::ParserInput;
use super::expression::read_expression;

/// Parse `{expression}` tag.
pub fn expression_tag_parser(parser_input: &mut ParserInput) -> Result<FragmentNode> {
    let start = parser_input.current_token_start();
    let expression = read_expression(parser_input)?;
    let end = parser_input.previous_token_end();

    Ok(FragmentNode::ExpressionTag(ExpressionTag {
        span: Span::new(start, end),
        expression,
    }))
}
