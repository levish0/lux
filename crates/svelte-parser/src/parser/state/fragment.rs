use crate::parser::state::element::element;
use crate::parser::state::tag::tag;
use crate::parser::state::text::text;
use super::super::Parser;

/// Fragment state dispatcher.
/// Matches reference: `state/fragment.js`
///
/// Dispatches to element (if `<`), tag (if `{`), or text.
pub fn fragment<'a>(parser: &mut Parser<'a>) {
    if parser.match_str("<") {
        element(parser);
    } else if parser.match_str("{") {
        tag(parser);
    } else {
        text(parser);
    }
}
