use winnow::error::ContextError;
use winnow::prelude::*;
use winnow::token::{literal, take_while};

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

/// Consume `{/keyword}` block closing delimiter.
pub fn eat_block_close<'a>(input: &mut Input<'a>, keyword: &str) -> winnow::Result<()> {
    literal("{").parse_next(input)?;
    skip_whitespace(input);
    literal("/").parse_next(input)?;
    skip_whitespace(input);
    literal(keyword).parse_next(input)?;
    skip_whitespace(input);
    literal("}").parse_next(input)?;
    Ok(())
}

/// Check if next tokens form `{:keyword` (without consuming).
pub fn at_block_continuation(input: &Input<'_>, keyword: &str) -> bool {
    let remaining: &str = &input.input;
    if let Some(rest) = remaining.strip_prefix('{') {
        let rest = rest.trim_start();
        if let Some(rest) = rest.strip_prefix(':') {
            let rest = rest.trim_start();
            rest.starts_with(keyword)
        } else {
            false
        }
    } else {
        false
    }
}

/// Consume `{:keyword` (opening brace, colon, keyword — but NOT closing `}`).
pub fn eat_block_continuation<'a>(input: &mut Input<'a>, keyword: &str) -> winnow::Result<()> {
    literal("{").parse_next(input)?;
    skip_whitespace(input);
    literal(":").parse_next(input)?;
    skip_whitespace(input);
    literal(keyword).parse_next(input)?;
    Ok(())
}
