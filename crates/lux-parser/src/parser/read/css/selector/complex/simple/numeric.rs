use lux_ast::css::selector::{Nth, Percentage, RelativeSelector, SimpleSelector};

use super::super::super::super::parser::CssParser;
use super::super::super::nth::{is_nth_start, is_percentage_start, read_nth, read_percentage};

pub(super) fn parse_numeric_selector<'a>(
    parser: &mut CssParser<'a>,
    inside_pseudo_class: bool,
    relative_selector: &mut RelativeSelector<'a>,
) -> bool {
    let start = parser.index;

    if inside_pseudo_class && is_nth_start(parser) {
        let value = read_nth(parser);
        relative_selector.selectors.push(SimpleSelector::Nth(Nth {
            span: parser.span(start, parser.index),
            value,
        }));
        return true;
    }

    if is_percentage_start(parser) {
        let value = read_percentage(parser);
        relative_selector
            .selectors
            .push(SimpleSelector::Percentage(Percentage {
                span: parser.span(start, parser.index),
                value,
            }));
        return true;
    }

    false
}
