use lux_ast::common::Span;
use lux_ast::template::root::FragmentNode;
use lux_ast::template::tag::RenderTag;
use winnow::Result;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::literal;

use crate::input::Input;
use crate::parser::read::expression::read_expression;
use crate::parser::utils::helpers::{require_whitespace, skip_whitespace};

/// Parse `{@render snippet(...)}`.
/// Assumes `{` already consumed. Starts at `@render`.
pub fn parse_render_tag<'a>(input: &mut Input<'a>, start: usize) -> Result<FragmentNode<'a>> {
    literal("@render").parse_next(input)?;
    require_whitespace(input)?;

    let expression = read_expression(input)?;
    skip_whitespace(input);
    literal("}").parse_next(input)?;

    let end = input.previous_token_end();

    Ok(FragmentNode::RenderTag(RenderTag {
        span: Span::new(start as u32, end as u32),
        expression,
        metadata: None,
    }))
}
