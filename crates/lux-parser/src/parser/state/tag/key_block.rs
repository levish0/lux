use lux_ast::common::Span;
use lux_ast::template::block::KeyBlock;
use lux_ast::template::root::FragmentNode;
use winnow::Result;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::literal;

use crate::input::Input;
use crate::parser::read::expression::read_expression;
use crate::parser::state::fragment::parse_block_fragment;
use crate::parser::utils::helpers::{eat_block_close, require_whitespace, skip_whitespace};

/// Parse `{#key expression}...{/key}`.
/// Assumes `{` and `#` already consumed.
pub fn parse_key_block<'a>(input: &mut Input<'a>, start: usize) -> Result<FragmentNode<'a>> {
    literal("key").parse_next(input)?;
    require_whitespace(input)?;

    let expression = read_expression(input)?;
    skip_whitespace(input);
    literal("}").parse_next(input)?;

    let fragment = parse_block_fragment(input)?;

    eat_block_close(input, "key")?;
    let end = input.previous_token_end();

    Ok(FragmentNode::KeyBlock(KeyBlock {
        span: Span::new(start as u32, end as u32),
        expression,
        fragment,
    }))
}
