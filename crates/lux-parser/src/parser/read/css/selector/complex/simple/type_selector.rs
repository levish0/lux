use lux_ast::css::selector::{
    ClassSelector, IdSelector, NestingSelector, RelativeSelector, SimpleSelector, TypeSelector,
};
use winnow::Result;

use super::super::super::super::parser::CssParser;
use super::super::super::combinator::is_combinator;

pub(super) fn parse_nesting_selector(
    parser: &mut CssParser<'_>,
    relative_selector: &mut RelativeSelector<'_>,
) -> bool {
    let start = parser.index;
    if !parser.eat("&") {
        return false;
    }

    relative_selector
        .selectors
        .push(SimpleSelector::NestingSelector(NestingSelector {
            span: parser.span(start, parser.index),
        }));
    true
}

pub(super) fn parse_universal_selector<'a>(
    parser: &mut CssParser<'a>,
    relative_selector: &mut RelativeSelector<'a>,
) -> Result<bool> {
    let start = parser.index;
    if !parser.eat("*") {
        return Ok(false);
    }

    let mut name = "*";
    if parser.eat("|") {
        name = parser.read_identifier()?;
    }
    relative_selector
        .selectors
        .push(SimpleSelector::TypeSelector(TypeSelector {
            span: parser.span(start, parser.index),
            name,
        }));
    Ok(true)
}

pub(super) fn parse_id_selector<'a>(
    parser: &mut CssParser<'a>,
    relative_selector: &mut RelativeSelector<'a>,
) -> Result<bool> {
    let start = parser.index;
    if !parser.eat("#") {
        return Ok(false);
    }

    let name = parser.read_identifier()?;
    relative_selector
        .selectors
        .push(SimpleSelector::IdSelector(IdSelector {
            span: parser.span(start, parser.index),
            name,
        }));
    Ok(true)
}

pub(super) fn parse_class_selector<'a>(
    parser: &mut CssParser<'a>,
    relative_selector: &mut RelativeSelector<'a>,
) -> Result<bool> {
    let start = parser.index;
    if !parser.eat(".") {
        return Ok(false);
    }

    let name = parser.read_identifier()?;
    relative_selector
        .selectors
        .push(SimpleSelector::ClassSelector(ClassSelector {
            span: parser.span(start, parser.index),
            name,
        }));
    Ok(true)
}

pub(super) fn parse_type_selector<'a>(
    parser: &mut CssParser<'a>,
    relative_selector: &mut RelativeSelector<'a>,
) -> Result<bool> {
    if is_combinator(parser) {
        return Ok(false);
    }

    let start = parser.index;
    let mut name = parser.read_identifier()?;
    if parser.eat("|") {
        name = parser.read_identifier()?;
    }
    relative_selector
        .selectors
        .push(SimpleSelector::TypeSelector(TypeSelector {
            span: parser.span(start, parser.index),
            name,
        }));

    Ok(true)
}
