mod attribute;
mod combinator;
mod complex;
mod nth;

use lux_ast::css::selector::SelectorList;
use winnow::Result;
use winnow::error::ContextError;

use self::complex::parse_selector;
use super::parser::CssParser;

pub fn parse_selector_list<'a>(
    parser: &mut CssParser<'a>,
    inside_pseudo_class: bool,
) -> Result<SelectorList<'a>> {
    let mut children = Vec::new();

    parser.skip_ws_and_comments();
    let start = parser.index;

    while !parser.at_end() {
        children.push(parse_selector(parser, inside_pseudo_class)?);
        let end = parser.index;

        parser.skip_ws_and_comments();

        let done = if inside_pseudo_class {
            parser.matches(")")
        } else {
            parser.matches("{")
        };

        if done {
            return Ok(SelectorList {
                span: parser.span(start, end),
                children,
            });
        }

        parser.eat_required(",")?;
        parser.skip_ws_and_comments();
    }

    Err(ContextError::new())
}
