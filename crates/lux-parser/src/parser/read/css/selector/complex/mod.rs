mod relative;
mod simple;

use lux_ast::css::selector::{ComplexSelector, RelativeSelector};
use winnow::Result;
use winnow::error::ContextError;

use self::relative::{is_selector_terminator, new_relative_selector};
use self::simple::parse_simple_selector;
use super::super::parser::CssParser;
use super::combinator::read_combinator;

pub fn parse_selector<'a>(
    parser: &mut CssParser<'a>,
    inside_pseudo_class: bool,
) -> Result<ComplexSelector<'a>> {
    let list_start = parser.index;
    let mut children: Vec<RelativeSelector<'a>> = Vec::new();
    let mut relative_selector_start = parser.index;
    let mut relative_selector = new_relative_selector(None, relative_selector_start);

    while !parser.at_end() {
        parse_simple_selector(parser, inside_pseudo_class, &mut relative_selector)?;

        let index = parser.index;
        parser.skip_ws_and_comments();

        if is_selector_terminator(parser, inside_pseudo_class) {
            parser.index = index;
            relative_selector.span = parser.span(relative_selector_start, index);
            children.push(relative_selector);

            return Ok(ComplexSelector {
                span: parser.span(list_start, index),
                children,
            });
        }

        parser.index = index;
        if let Some(combinator) = read_combinator(parser) {
            if !relative_selector.selectors.is_empty() {
                relative_selector.span = parser.span(relative_selector_start, index);
                children.push(relative_selector);
            }

            let comb_start = combinator.span.start as usize - parser.offset as usize;
            relative_selector_start = comb_start;
            relative_selector = new_relative_selector(Some(combinator), comb_start);

            parser.skip_whitespace();

            if is_selector_terminator(parser, inside_pseudo_class) {
                return Err(ContextError::new());
            }
        }
    }

    Err(ContextError::new())
}
