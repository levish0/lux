use winnow::error::ContextError;
use winnow::prelude::*;
use winnow::token::take_while;

use crate::input::Input;

/// Skip optional whitespace.
pub fn skip_whitespace(input: &mut Input<'_>) {
    let _: Result<&str, ContextError> =
        take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(input);
}

/// Require at least one whitespace character.
pub fn require_whitespace(input: &mut Input<'_>) -> winnow::Result<()> {
    take_while(1.., |c: char| c.is_ascii_whitespace())
        .void()
        .parse_next(input)
}

