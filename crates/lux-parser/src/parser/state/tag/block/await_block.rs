use lux_ast::common::Span;
use lux_ast::template::block::AwaitBlock;
use lux_ast::template::root::FragmentNode;
use winnow::Result;
use winnow::combinator::opt;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::literal;

use crate::input::Input;
use crate::parser::read::expression::read_expression;
use crate::parser::state::fragment::parse_block_fragment;
use crate::parser::utils::helpers::{
    at_block_continuation, eat_block_close, eat_block_continuation, require_whitespace,
    skip_whitespace,
};

/// Parse `{#await expression}...{:then value}...{:catch error}...{/await}`.
/// Assumes `{` and `#` already consumed.
pub fn parse_await_block<'a>(input: &mut Input<'a>, start: usize) -> Result<FragmentNode<'a>> {
    literal("await").parse_next(input)?;
    require_whitespace(input)?;

    let expression = read_expression(input)?;
    skip_whitespace(input);

    let mut value = None;
    let mut error = None;
    let mut pending = None;
    let mut then = None;
    let mut catch = None;

    // Inline `then` or `catch` in the opening tag
    if opt(literal("then")).parse_next(input)?.is_some() {
        let remaining: &str = &input.input;
        if !remaining.trim_start().starts_with('}') {
            require_whitespace(input)?;
            value = Some(read_expression(input)?);
            skip_whitespace(input);
        } else {
            skip_whitespace(input);
        }
        literal("}").parse_next(input)?;
        then = Some(parse_block_fragment(input)?);
    } else if opt(literal("catch")).parse_next(input)?.is_some() {
        let remaining: &str = &input.input;
        if !remaining.trim_start().starts_with('}') {
            require_whitespace(input)?;
            error = Some(read_expression(input)?);
            skip_whitespace(input);
        } else {
            skip_whitespace(input);
        }
        literal("}").parse_next(input)?;
        catch = Some(parse_block_fragment(input)?);
    } else {
        literal("}").parse_next(input)?;
        pending = Some(parse_block_fragment(input)?);
    }

    // Continuation clauses
    if then.is_none() && at_block_continuation(input, "then") {
        eat_block_continuation(input, "then")?;
        let remaining: &str = &input.input;
        if !remaining.trim_start().starts_with('}') {
            require_whitespace(input)?;
            value = Some(read_expression(input)?);
            skip_whitespace(input);
        } else {
            skip_whitespace(input);
        }
        literal("}").parse_next(input)?;
        then = Some(parse_block_fragment(input)?);
    }

    if catch.is_none() && at_block_continuation(input, "catch") {
        eat_block_continuation(input, "catch")?;
        let remaining: &str = &input.input;
        if !remaining.trim_start().starts_with('}') {
            require_whitespace(input)?;
            error = Some(read_expression(input)?);
            skip_whitespace(input);
        } else {
            skip_whitespace(input);
        }
        literal("}").parse_next(input)?;
        catch = Some(parse_block_fragment(input)?);
    }

    eat_block_close(input, "await")?;
    let end = input.previous_token_end();

    Ok(FragmentNode::AwaitBlock(AwaitBlock {
        span: Span::new(start as u32, end as u32),
        expression,
        value,
        error,
        pending,
        then,
        catch,
    }))
}
