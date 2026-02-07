mod await_block;
mod const_tag;
mod debug_tag;
mod each_block;
mod expression_tag;
mod html_tag;
mod if_block;
mod key_block;
mod render_tag;
mod snippet_block;

use lux_ast::template::root::FragmentNode;
use winnow::Result;
use winnow::combinator::{dispatch, fail, peek};
use winnow::error::ContextError;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::{any, literal};

use crate::input::Input;
use crate::parser::utils::helpers::skip_whitespace;

/// Parse a template tag: `{...}`.
pub fn parse_tag<'a>(input: &mut Input<'a>) -> Result<FragmentNode<'a>> {
    let start = input.current_token_start();

    literal("{").parse_next(input)?;
    skip_whitespace(input);

    dispatch! {peek(any);
        '#' => |i: &mut Input<'a>| parse_block_open(i, start),
        ':' | '/' => fail,
        '@' => |i: &mut Input<'a>| parse_special(i, start),
        _ => |i: &mut Input<'a>| expression_tag::parse_expression_tag(i, start),
    }
    .parse_next(input)
}

fn parse_block_open<'a>(input: &mut Input<'a>, start: usize) -> Result<FragmentNode<'a>> {
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

fn parse_special<'a>(input: &mut Input<'a>, start: usize) -> Result<FragmentNode<'a>> {
    let remaining: &str = &input.input;
    if remaining.starts_with("@html") {
        html_tag::parse_html_tag(input, start)
    } else if remaining.starts_with("@const") {
        const_tag::parse_const_tag(input, start)
    } else if remaining.starts_with("@debug") {
        debug_tag::parse_debug_tag(input, start)
    } else if remaining.starts_with("@render") {
        render_tag::parse_render_tag(input, start)
    } else {
        Err(ContextError::new())
    }
}
