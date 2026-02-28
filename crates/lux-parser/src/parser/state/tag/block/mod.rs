mod await_block;
mod each_block;
mod if_block;
mod key_block;
mod snippet_block;

use lux_ast::template::root::FragmentNode;
use winnow::Parser;
use winnow::Result;
use winnow::error::ContextError;
use winnow::token::literal;

use crate::input::Input;

pub fn parse_block_open<'a>(input: &mut Input<'a>, start: usize) -> Result<FragmentNode<'a>> {
    literal("#").parse_next(input)?;

    let remaining: &str = &input.input;
    if remaining.starts_with("if") {
        if_block::parse_if_block(input, start)
    } else if remaining.starts_with("each") {
        each_block::parse_each_block(input, start)
    } else if remaining.starts_with("await") {
        await_block::parse_await_block(input, start)
    } else if remaining.starts_with("key") {
        key_block::parse_key_block(input, start)
    } else if remaining.starts_with("snippet") {
        snippet_block::parse_snippet_block(input, start)
    } else {
        Err(ContextError::new())
    }
}
