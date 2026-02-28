use lux_ast::css::selector::{AttributeSelector, RelativeSelector, SimpleSelector};
use winnow::Result;

use super::super::super::super::parser::CssParser;
use super::super::super::attribute::{read_attr_flags, read_matcher};

pub(super) fn parse_attribute_selector<'a>(
    parser: &mut CssParser<'a>,
    relative_selector: &mut RelativeSelector<'a>,
) -> Result<bool> {
    let start = parser.index;
    if !parser.eat("[") {
        return Ok(false);
    }

    parser.skip_whitespace();
    let name = parser.read_identifier()?;
    parser.skip_whitespace();

    let matcher = read_matcher(parser);
    let value = if matcher.is_some() {
        parser.skip_whitespace();
        Some(parser.read_attribute_value())
    } else {
        None
    };

    parser.skip_whitespace();
    let flags = read_attr_flags(parser);
    parser.skip_whitespace();
    parser.eat_required("]")?;

    relative_selector
        .selectors
        .push(SimpleSelector::AttributeSelector(AttributeSelector {
            span: parser.span(start, parser.index),
            name,
            matcher,
            value,
            flags,
        }));

    Ok(true)
}
