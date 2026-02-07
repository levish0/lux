mod expression_tag;
mod html_tag;

use lux_ast::template::root::FragmentNode;
use winnow::Result;
use winnow::combinator::{dispatch, fail, peek};
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

fn parse_block_open<'a>(_input: &mut Input<'a>, _start: usize) -> Result<FragmentNode<'a>> {
    // TODO: {#if}, {#each}, {#await}, {#key}, {#snippet}
    fail::<Input<'_>, FragmentNode<'_>, _>.parse_next(_input)
}

fn parse_special<'a>(input: &mut Input<'a>, start: usize) -> Result<FragmentNode<'a>> {
    let remaining: &str = &input.input;
    if remaining.starts_with("@html") {
        html_tag::parse_html_tag(input, start)
    } else {
        // TODO: @const, @debug, @render, @attach
        fail.parse_next(input)
    }
}
