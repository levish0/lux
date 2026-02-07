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

/// Peek at the next byte without consuming.
pub fn peek_byte(input: &Input<'_>) -> Option<u8> {
    let s: &str = &(*input.input);
    s.as_bytes().first().copied()
}

/// Check if input starts with a given string.
pub fn peek_str(input: &Input<'_>, s: &str) -> bool {
    let remaining: &str = &(*input.input);
    remaining.starts_with(s)
}

/// Get remaining input as &str.
pub fn remaining_input<'a>(input: &Input<'a>) -> &'a str {
    let s: &str = &(*input.input);
    s
}
