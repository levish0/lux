use lux_ast::css::selector::{
    AttributeSelector, ClassSelector, IdSelector, NestingSelector, Nth, Percentage,
    PseudoClassSelector, PseudoElementSelector, RelativeSelector, SimpleSelector, TypeSelector,
};
use winnow::Result;

use super::super::super::parser::CssParser;
use super::super::attribute::{read_attr_flags, read_matcher};
use super::super::combinator::is_combinator;
use super::super::nth::{is_nth_start, is_percentage_start, read_nth, read_percentage};
use super::super::parse_selector_list;

pub(super) fn parse_simple_selector<'a>(
    parser: &mut CssParser<'a>,
    inside_pseudo_class: bool,
    relative_selector: &mut RelativeSelector<'a>,
) -> Result<()> {
    let start = parser.index;

    if parser.eat("&") {
        relative_selector
            .selectors
            .push(SimpleSelector::NestingSelector(NestingSelector {
                span: parser.span(start, parser.index),
            }));
        return Ok(());
    }

    if parser.eat("*") {
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
        return Ok(());
    }

    if parser.eat("#") {
        let name = parser.read_identifier()?;
        relative_selector
            .selectors
            .push(SimpleSelector::IdSelector(IdSelector {
                span: parser.span(start, parser.index),
                name,
            }));
        return Ok(());
    }

    if parser.eat(".") {
        let name = parser.read_identifier()?;
        relative_selector
            .selectors
            .push(SimpleSelector::ClassSelector(ClassSelector {
                span: parser.span(start, parser.index),
                name,
            }));
        return Ok(());
    }

    if parser.eat("::") {
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
        return Ok(());
    }

    if parser.eat(":") {
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
        return Ok(());
    }

    if parser.eat("[") {
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
        return Ok(());
    }

    if inside_pseudo_class && is_nth_start(parser) {
        let value = read_nth(parser);
        relative_selector.selectors.push(SimpleSelector::Nth(Nth {
            span: parser.span(start, parser.index),
            value,
        }));
        return Ok(());
    }

    if is_percentage_start(parser) {
        let value = read_percentage(parser);
        relative_selector
            .selectors
            .push(SimpleSelector::Percentage(Percentage {
                span: parser.span(start, parser.index),
                value,
            }));
        return Ok(());
    }

    if is_combinator(parser) {
        return Ok(());
    }

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

    Ok(())
}
