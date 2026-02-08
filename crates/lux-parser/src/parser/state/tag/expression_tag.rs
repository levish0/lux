use lux_ast::common::Span;
use lux_ast::template::root::FragmentNode;
use lux_ast::template::tag::ExpressionTag;
use winnow::Result;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::literal;

use crate::input::Input;
use crate::parser::read::expression::read_expression;
use crate::parser::utils::helpers::skip_whitespace;

/// Parse `{expression}`.
/// Assumes `{` already consumed and whitespace skipped.
pub fn parse_expression_tag<'a>(input: &mut Input<'a>, start: usize) -> Result<FragmentNode<'a>> {
    let expression = read_expression(input)?;
    skip_whitespace(input);
    literal("}").parse_next(input)?;

    let end = input.previous_token_end();

    Ok(FragmentNode::ExpressionTag(ExpressionTag {
        span: Span::new(start as u32, end as u32),
        expression,
    }))
}
