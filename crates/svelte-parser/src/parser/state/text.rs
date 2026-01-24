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

    while parser.index < parser.template.len() && !parser.match_str("<") && !parser.match_str("{") {
        parser.index += 1;
    }

    let raw = &parser.template[start..parser.index];
    let data = decode_character_references(raw, false);

    parser.append(FragmentNode::Text(Text {
        span: Span::new(start, parser.index),
        raw: raw.to_string(),
        data,
    }));
}
