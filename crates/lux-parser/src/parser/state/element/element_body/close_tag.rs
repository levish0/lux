use winnow::Result;
use winnow::prelude::*;
use winnow::token::{literal, take_while};

use crate::input::Input;
use crate::parser::state::element::attribute::is_tag_name_char;
use crate::parser::utils::helpers::skip_whitespace;

pub(super) fn maybe_eat_matching_closing_tag(input: &mut Input<'_>, name: &str) -> Result<()> {
    // Graceful closing: consume </name> only if it matches.
    let remaining: &str = &input.input;
    if let Some(stripped) = remaining.strip_prefix("</") {
        let after_slash = stripped.trim_start();
        let name_end = after_slash
            .find(|c: char| !is_tag_name_char(c))
            .unwrap_or(after_slash.len());

        if &after_slash[..name_end] == name {
            eat_closing_tag(input)?;
        }
        // else: ancestor closing tag -> don't consume (auto-closed)
    }
    // else: sibling opening tag caused auto-close -> don't consume
    Ok(())
}

pub(super) fn eat_closing_tag(input: &mut Input<'_>) -> Result<()> {
    literal("</").parse_next(input)?;
    skip_whitespace(input);
    let _: &str = take_while(1.., is_tag_name_char).parse_next(input)?;
    skip_whitespace(input);
    literal(">").parse_next(input)?;
    Ok(())
}
