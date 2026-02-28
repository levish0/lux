use lux_ast::css::selector::{
    AttributeSelector, ClassSelector, ComplexSelector, IdSelector, NestingSelector, Nth,
    Percentage, PseudoClassSelector, PseudoElementSelector, RelativeSelector, SimpleSelector,
    TypeSelector,
};
use winnow::Result;
use winnow::error::ContextError;

use super::super::parser::CssParser;
use super::attribute::{read_attr_flags, read_matcher};
use super::combinator::{is_combinator, read_combinator};
use super::nth::{is_nth_start, is_percentage_start, read_nth, read_percentage};
use super::parse_selector_list;

pub fn parse_selector<'a>(
    parser: &mut CssParser<'a>,
    inside_pseudo_class: bool,
) -> Result<ComplexSelector<'a>> {
    let list_start = parser.index;
    let mut children: Vec<RelativeSelector<'a>> = Vec::new();
    let mut relative_selector_start = parser.index;
    let mut relative_selector = new_relative_selector(None, relative_selector_start);

    while !parser.at_end() {
        let start = parser.index;

        if parser.eat("&") {
            relative_selector
                .selectors
                .push(SimpleSelector::NestingSelector(NestingSelector {
                    span: parser.span(start, parser.index),
                }));
        } else if parser.eat("*") {
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
        } else if parser.eat("#") {
            let name = parser.read_identifier()?;
            relative_selector
                .selectors
                .push(SimpleSelector::IdSelector(IdSelector {
                    span: parser.span(start, parser.index),
                    name,
                }));
        } else if parser.eat(".") {
            let name = parser.read_identifier()?;
            relative_selector
                .selectors
                .push(SimpleSelector::ClassSelector(ClassSelector {
                    span: parser.span(start, parser.index),
                    name,
                }));
        } else if parser.eat("::") {
            let name = parser.read_identifier()?;
            let end = parser.index;
            // Read and discard inner selectors of pseudo element
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
        } else if parser.eat(":") {
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
        } else if parser.eat("[") {
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
        } else if inside_pseudo_class && is_nth_start(parser) {
            let value = read_nth(parser);
            relative_selector.selectors.push(SimpleSelector::Nth(Nth {
                span: parser.span(start, parser.index),
                value,
            }));
        } else if is_percentage_start(parser) {
            let value = read_percentage(parser);
            relative_selector
                .selectors
                .push(SimpleSelector::Percentage(Percentage {
                    span: parser.span(start, parser.index),
                    value,
                }));
        } else if !is_combinator(parser) {
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
        }

        let index = parser.index;
        parser.skip_ws_and_comments();

        let done = if inside_pseudo_class {
            parser.matches(",") || parser.matches(")")
        } else {
            parser.matches(",") || parser.matches("{")
        };

        if done {
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

            let bad = if inside_pseudo_class {
                parser.matches(",") || parser.matches(")")
            } else {
                parser.matches(",") || parser.matches("{")
            };
            if bad {
                return Err(ContextError::new());
            }
        }
    }

    Err(ContextError::new())
}

fn new_relative_selector(
    combinator: Option<lux_ast::css::selector::Combinator>,
    start: usize,
) -> RelativeSelector<'static> {
    RelativeSelector {
        span: lux_ast::common::Span::new(start as u32, 0),
        combinator,
        selectors: Vec::new(),
    }
}
