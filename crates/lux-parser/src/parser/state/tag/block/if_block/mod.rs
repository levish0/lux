use lux_ast::common::Span;
use lux_ast::template::block::IfBlock;
use lux_ast::template::root::FragmentNode;
use winnow::Result;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::literal;

use crate::input::Input;
use crate::parser::read::expression::read_expression_until;
use crate::parser::state::fragment::parse_block_fragment;
use crate::parser::utils::helpers::{eat_block_close, require_whitespace, skip_whitespace};

mod alternate;

use alternate::parse_if_alternate;

/// Parse `{#if test}...{:else if test2}...{:else}...{/if}`.
/// Assumes `{` and `#` are already consumed.
pub fn parse_if_block<'a>(input: &mut Input<'a>, start: usize) -> Result<FragmentNode<'a>> {
    literal("if").parse_next(input)?;
    require_whitespace(input)?;

    let test = read_expression_until(input, b"")?;
    skip_whitespace(input);
    literal("}").parse_next(input)?;

    let consequent = parse_block_fragment(input)?;
    let mut alternate = parse_if_alternate(input)?;

    eat_block_close(input, "if")?;
    let end = input.previous_token_end();
    alternate::set_elseif_span_end(&mut alternate, end as u32);

    Ok(FragmentNode::IfBlock(IfBlock {
        span: Span::new(start as u32, end as u32),
        elseif: false,
        test,
        consequent,
        alternate,
    }))
}
