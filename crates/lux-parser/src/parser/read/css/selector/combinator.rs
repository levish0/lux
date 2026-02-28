use lux_ast::css::selector::{Combinator, CombinatorKind};

use super::super::parser::CssParser;

pub fn read_combinator(parser: &mut CssParser<'_>) -> Option<Combinator> {
    let start = parser.index;
    parser.skip_whitespace();

    let index = parser.index;

    let kind = if parser.eat("||") {
        Some(CombinatorKind::Column)
    } else if parser.eat(">") {
        Some(CombinatorKind::Child)
    } else if parser.eat("+") {
        Some(CombinatorKind::NextSibling)
    } else if parser.eat("~") {
        Some(CombinatorKind::SubsequentSibling)
    } else {
        None
    };

    if let Some(kind) = kind {
        let end = parser.index;
        parser.skip_whitespace();
        return Some(Combinator {
            span: parser.span(index, end),
            kind,
        });
    }

    // Whitespace-only combinator (descendant)
    if parser.index != start {
        return Some(Combinator {
            span: parser.span(start, parser.index),
            kind: CombinatorKind::Descendant,
        });
    }

    None
}

pub fn is_combinator(parser: &CssParser<'_>) -> bool {
    matches!(parser.peek(), Some(b'+' | b'~' | b'>' | b'|'))
}
