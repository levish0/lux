use lux_ast::common::Span;
use lux_ast::css::selector::{Combinator, RelativeSelector};

use super::super::super::parser::CssParser;

pub(super) fn new_relative_selector<'a>(
    combinator: Option<Combinator>,
    start: usize,
) -> RelativeSelector<'a> {
    RelativeSelector {
        span: Span::new(start as u32, 0),
        combinator,
        selectors: Vec::new(),
    }
}

pub(super) fn is_selector_terminator(parser: &CssParser<'_>, inside_pseudo_class: bool) -> bool {
    if inside_pseudo_class {
        parser.matches(",") || parser.matches(")")
    } else {
        parser.matches(",") || parser.matches("{")
    }
}
