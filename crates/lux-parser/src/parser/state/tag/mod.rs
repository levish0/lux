use winnow::stream::Location;
mod block;
mod expression;
mod special;

use lux_ast::template::root::FragmentNode;
use winnow::Result;
use winnow::combinator::{dispatch, fail, peek};
use winnow::prelude::*;
use winnow::token::{any, literal};

use crate::input::Input;
use crate::parser::utils::helpers::skip_whitespace;

/// Parse a template tag: `{...}`.
pub fn parse_tag<'a>(input: &mut Input<'a>) -> Result<FragmentNode<'a>> {
    let start = input.current_token_start();

    literal("{").parse_next(input)?;
    skip_whitespace(input);

    dispatch! {peek(any);
        '#' => |i: &mut Input<'a>| block::parse_block_open(i, start),
        ':' | '/' => fail,
        '@' => |i: &mut Input<'a>| special::parse_special(i, start),
        _ => |i: &mut Input<'a>| expression::parse_expression_tag(i, start),
    }
    .parse_next(input)
}
