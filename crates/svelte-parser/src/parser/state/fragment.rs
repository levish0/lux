use super::super::{ParseError, Parser};
use crate::parser::state::element::element;
use crate::parser::state::tag::tag;
use crate::parser::state::text::text;

/// Fragment state dispatcher.
/// Matches reference: `state/fragment.js`
///
/// Dispatches to element (if `<`), tag (if `{`), or text.
pub fn fragment(parser: &mut Parser) -> Result<(), ParseError> {
    if parser.match_str("<") {
        element(parser)?;
    } else if parser.match_str("{") {
        tag(parser)?;
    } else {
        text(parser);
    }
    Ok(())
}
