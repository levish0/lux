use lux_ast::css::selector::{
    PseudoClassSelector, PseudoElementSelector, RelativeSelector, SimpleSelector,
};
use winnow::Result;

use super::super::super::super::parser::CssParser;
use super::super::super::parse_selector_list;

pub(super) fn parse_pseudo_element_selector<'a>(
    parser: &mut CssParser<'a>,
    relative_selector: &mut RelativeSelector<'a>,
) -> Result<bool> {
    let start = parser.index;
    if !parser.eat("::") {
        return Ok(false);
    }

    let name = parser.read_identifier()?;
    let end = parser.index;

    // Read and discard inner selectors of pseudo element.
    if parser.eat("(") {
        let _ = parse_selector_list(parser, true);
        parser.eat_required(")")?;
    }

    relative_selector
        .selectors
        .push(SimpleSelector::PseudoElementSelector(
            PseudoElementSelector {
                span: parser.span(start, end),
                name,
            },
        ));

    Ok(true)
}

pub(super) fn parse_pseudo_class_selector<'a>(
    parser: &mut CssParser<'a>,
    relative_selector: &mut RelativeSelector<'a>,
) -> Result<bool> {
    let start = parser.index;
    if !parser.eat(":") {
        return Ok(false);
    }

    let name = parser.read_identifier()?;
    let args = if parser.eat("(") {
        let list = parse_selector_list(parser, true)?;
        parser.eat_required(")")?;
        Some(list)
    } else {
        None
    };

    relative_selector
        .selectors
        .push(SimpleSelector::PseudoClassSelector(PseudoClassSelector {
            span: parser.span(start, parser.index),
            name,
            args,
        }));

    Ok(true)
}
