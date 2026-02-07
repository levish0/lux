use lux_ast::common::Span;
use lux_ast::template::block::EachBlock;
use lux_ast::template::root::FragmentNode;
use winnow::Result;
use winnow::combinator::opt;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::{literal, take_while};

use crate::input::Input;
use crate::parser::read::expression::{read_each_expression, read_expression_until};
use crate::parser::state::fragment::parse_block_fragment;
use crate::parser::utils::helpers::{
    at_block_continuation, eat_block_close, eat_block_continuation, require_whitespace,
    skip_whitespace,
};

/// Parse `{#each expression as context, index (key)}...{:else}...{/each}`.
/// Assumes `{` and `#` already consumed.
pub fn parse_each_block<'a>(input: &mut Input<'a>, start: usize) -> Result<FragmentNode<'a>> {
    literal("each").parse_next(input)?;
    require_whitespace(input)?;

    // Read each expression — OXC grammar-based, with ` as ` fallback
    let expression = read_each_expression(input)?;
    skip_whitespace(input);

    // Optional `as context`
    let context = if opt(literal("as")).parse_next(input)?.is_some() {
        require_whitespace(input)?;
        Some(read_expression_until(input, b",(")?)
    } else {
        None
    };

    skip_whitespace(input);

    // Optional `, index`
    let index = if opt(literal(",")).parse_next(input)?.is_some() {
        skip_whitespace(input);
        let name: &str = take_while(1.., |c: char| {
            c.is_ascii_alphanumeric() || c == '_' || c == '$'
        })
        .parse_next(input)?;
        skip_whitespace(input);
        Some(name)
    } else {
        None
    };

    // Optional `(key)`
    let key = if opt(literal("(")).parse_next(input)?.is_some() {
        skip_whitespace(input);
        let key_expr = read_expression_until(input, b")")?;
        skip_whitespace(input);
        literal(")").parse_next(input)?;
        skip_whitespace(input);
        Some(key_expr)
    } else {
        None
    };

    literal("}").parse_next(input)?;

    let body = parse_block_fragment(input)?;

    // Optional {:else} fallback
    let fallback = if at_block_continuation(input, "else") {
        eat_block_continuation(input, "else")?;
        skip_whitespace(input);
        literal("}").parse_next(input)?;
        Some(parse_block_fragment(input)?)
    } else {
        None
    };

    eat_block_close(input, "each")?;
    let end = input.previous_token_end();

    Ok(FragmentNode::EachBlock(EachBlock {
        span: Span::new(start as u32, end as u32),
        expression,
        context,
        body,
        fallback,
        index,
        key,
        metadata: None,
    }))
}
