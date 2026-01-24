use std::borrow::Cow;

use svelte_ast::node::FragmentNode;
use svelte_ast::span::Span;
use svelte_ast::text::Text;

use super::super::Parser;
use super::super::html_entities::decode_character_references;

/// Text state.
/// Matches reference: `state/text.js`
///
/// Consumes characters until `<` or `{` is encountered.
pub fn text(parser: &mut Parser) {
    let start = parser.index;

    let raw = parser.read_until_char(|ch| ch == b'<' || ch == b'{');

    if raw.is_empty() {
        return;
    }
    let decoded = decode_character_references(raw, false);
    let data = match decoded {
        Cow::Borrowed(s) => s,
        Cow::Owned(s) => parser.allocator.alloc_str(&s),
    };

    parser.append(FragmentNode::Text(Text {
        span: Span::new(start, parser.index),
        raw,
        data,
    }));
}
