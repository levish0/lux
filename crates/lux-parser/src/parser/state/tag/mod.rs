mod expression_tag;
mod html_tag;

use lux_ast::template::root::FragmentNode;
use winnow::Result;
use winnow::combinator::fail;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::literal;

use crate::input::Input;
use crate::parser::utils::helpers::{peek_byte, skip_whitespace};

/// Parse a template tag: `{...}`.
pub fn parse_tag<'a>(input: &mut Input<'a>) -> Result<FragmentNode<'a>> {
    let start = input.current_token_start();

    literal("{").parse_next(input)?;
    skip_whitespace(input);

    let next = peek_byte(input);

    match next {
        Some(b'#') => {
            // TODO: block opening
            fail.parse_next(input)
        }
        Some(b':') => fail.parse_next(input),
        Some(b'/') => fail.parse_next(input),
        Some(b'@') => {
            let remaining: &str = &(*input.input);
            if remaining.starts_with("@html") {
                html_tag::parse_html_tag(input, start)
            } else {
                // TODO: @const, @debug, @render, @attach
                fail.parse_next(input)
            }
        }
        _ => expression_tag::parse_expression_tag(input, start),
    }
}
