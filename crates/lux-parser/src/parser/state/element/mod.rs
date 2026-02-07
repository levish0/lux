mod comment;
mod regular;

use lux_ast::template::root::FragmentNode;
use winnow::Result;
use winnow::error::ContextError;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::literal;

use crate::input::Input;

/// Parse an element: `<...>`.
pub fn parse_element<'a>(input: &mut Input<'a>) -> Result<FragmentNode<'a>> {
    let start = input.current_token_start();

    literal("<").parse_next(input)?;

    let remaining: &str = &(*input.input);

    if remaining.starts_with("!--") {
        return comment::parse_comment(input, start);
    }

    if remaining.starts_with("/") {
        return Err(ContextError::new());
    }

    regular::parse_opening_tag(input, start)
}
