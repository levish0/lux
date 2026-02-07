use lux_ast::css::selector::*;
use winnow::Result;
use winnow::error::ContextError;

use super::value::CssParser;

pub fn parse_selector_list<'a>(
    p: &mut CssParser<'a>,
    inside_pseudo_class: bool,
) -> Result<SelectorList<'a>> {
    let mut children = Vec::new();

    p.skip_ws_and_comments();
    let start = p.index;

    while !p.at_end() {
        children.push(parse_selector(p, inside_pseudo_class)?);
        let end = p.index;

        p.skip_ws_and_comments();

        let done = if inside_pseudo_class {
            p.matches(")")
        } else {
            p.matches("{")
        };

        if done {
            return Ok(SelectorList {
                span: p.span(start, end),
                children,
            });
        }

        p.eat_required(",")?;
        p.skip_ws_and_comments();
    }

    Err(ContextError::new())
}

fn parse_selector<'a>(
    p: &mut CssParser<'a>,
    inside_pseudo_class: bool,
) -> Result<ComplexSelector<'a>> {
    let list_start = p.index;
    let mut children: Vec<RelativeSelector<'a>> = Vec::new();
    let mut relative_selector = new_relative_selector(None, p.index);

    while !p.at_end() {
        let start = p.index;

        if p.eat("&") {
            relative_selector
                .selectors
                .push(SimpleSelector::NestingSelector(NestingSelector {
                    span: p.span(start, p.index),
                }));
        } else if p.eat("*") {
            let mut name = "*";
            if p.eat("|") {
                name = p.read_identifier()?;
            }
            relative_selector
                .selectors
                .push(SimpleSelector::TypeSelector(TypeSelector {
                    span: p.span(start, p.index),
                    name,
                }));
        } else if p.eat("#") {
            let name = p.read_identifier()?;
            relative_selector
                .selectors
                .push(SimpleSelector::IdSelector(IdSelector {
                    span: p.span(start, p.index),
                    name,
                }));
        } else if p.eat(".") {
            let name = p.read_identifier()?;
            relative_selector
                .selectors
                .push(SimpleSelector::ClassSelector(ClassSelector {
                    span: p.span(start, p.index),
                    name,
                }));
        } else if p.eat("::") {
            let name = p.read_identifier()?;
            let end = p.index;
            // Read and discard inner selectors of pseudo element
            if p.eat("(") {
                let _ = parse_selector_list(p, true);
                p.eat_required(")")?;
            }
            relative_selector
                .selectors
                .push(SimpleSelector::PseudoElementSelector(
                    PseudoElementSelector {
                        span: p.span(start, end),
                        name,
                    },
                ));
        } else if p.eat(":") {
            let name = p.read_identifier()?;
            let args = if p.eat("(") {
                let list = parse_selector_list(p, true)?;
                p.eat_required(")")?;
                Some(list)
            } else {
                None
            };
            relative_selector
                .selectors
                .push(SimpleSelector::PseudoClassSelector(PseudoClassSelector {
                    span: p.span(start, p.index),
                    name,
                    args,
                }));
        } else if p.eat("[") {
            p.skip_whitespace();
            let name = p.read_identifier()?;
            p.skip_whitespace();

            let matcher = read_matcher(p);
            let value = if matcher.is_some() {
                p.skip_whitespace();
                Some(p.read_attribute_value())
            } else {
                None
            };

            p.skip_whitespace();
            let flags = read_attr_flags(p);
            p.skip_whitespace();
            p.eat_required("]")?;

            relative_selector
                .selectors
                .push(SimpleSelector::AttributeSelector(AttributeSelector {
                    span: p.span(start, p.index),
                    name,
                    matcher,
                    value,
                    flags,
                }));
        } else if inside_pseudo_class && is_nth_start(p) {
            let value = read_nth(p);
            relative_selector.selectors.push(SimpleSelector::Nth(Nth {
                span: p.span(start, p.index),
                value,
            }));
        } else if is_percentage_start(p) {
            let value = read_percentage(p);
            relative_selector
                .selectors
                .push(SimpleSelector::Percentage(Percentage {
                    span: p.span(start, p.index),
                    value,
                }));
        } else if !is_combinator(p) {
            let mut name = p.read_identifier()?;
            if p.eat("|") {
                name = p.read_identifier()?;
            }
            relative_selector
                .selectors
                .push(SimpleSelector::TypeSelector(TypeSelector {
                    span: p.span(start, p.index),
                    name,
                }));
        }

        let index = p.index;
        p.skip_ws_and_comments();

        let done = if inside_pseudo_class {
            p.matches(",") || p.matches(")")
        } else {
            p.matches(",") || p.matches("{")
        };

        if done {
            p.index = index;
            relative_selector.span = p.span(
                relative_selector.span.start as usize - p.offset as usize,
                index,
            );
            children.push(relative_selector);

            return Ok(ComplexSelector {
                span: p.span(list_start, index),
                children,
                is_global: false,
                used: false,
            });
        }

        p.index = index;
        if let Some(combinator) = read_combinator(p) {
            if !relative_selector.selectors.is_empty() {
                relative_selector.span = p.span(
                    relative_selector.span.start as usize - p.offset as usize,
                    index,
                );
                children.push(relative_selector);
            }

            let comb_start = combinator.span.start as usize - p.offset as usize;
            relative_selector = new_relative_selector(Some(combinator), comb_start);

            p.skip_whitespace();

            let bad = if inside_pseudo_class {
                p.matches(",") || p.matches(")")
            } else {
                p.matches(",") || p.matches("{")
            };
            if bad {
                return Err(ContextError::new());
            }
        }
    }

    Err(ContextError::new())
}

fn new_relative_selector(
    combinator: Option<Combinator>,
    start: usize,
) -> RelativeSelector<'static> {
    RelativeSelector {
        span: lux_ast::common::Span::new(start as u32, 0),
        combinator,
        selectors: Vec::new(),
        is_global: false,
        is_global_like: false,
        scoped: false,
    }
}

fn read_combinator(p: &mut CssParser<'_>) -> Option<Combinator> {
    let start = p.index;
    p.skip_whitespace();

    let index = p.index;

    let kind = if p.eat("||") {
        Some(CombinatorKind::Column)
    } else if p.eat(">") {
        Some(CombinatorKind::Child)
    } else if p.eat("+") {
        Some(CombinatorKind::NextSibling)
    } else if p.eat("~") {
        Some(CombinatorKind::SubsequentSibling)
    } else {
        None
    };

    if let Some(kind) = kind {
        let end = p.index;
        p.skip_whitespace();
        return Some(Combinator {
            span: p.span(index, end),
            kind,
        });
    }

    // Whitespace-only combinator (descendant)
    if p.index != start {
        return Some(Combinator {
            span: p.span(start, p.index),
            kind: CombinatorKind::Descendant,
        });
    }

    None
}

fn is_combinator(p: &CssParser<'_>) -> bool {
    matches!(p.peek(), Some(b'+' | b'~' | b'>' | b'|'))
}

fn read_matcher<'a>(p: &mut CssParser<'a>) -> Option<&'a str> {
    let start = p.index;
    let remaining = p.remaining();

    if remaining.starts_with("~=")
        || remaining.starts_with("^=")
        || remaining.starts_with("$=")
        || remaining.starts_with("*=")
        || remaining.starts_with("|=")
    {
        p.index += 2;
        Some(&p.source[start..p.index])
    } else if remaining.starts_with('=') {
        p.index += 1;
        Some(&p.source[start..p.index])
    } else {
        None
    }
}

fn read_attr_flags<'a>(p: &mut CssParser<'a>) -> Option<&'a str> {
    let start = p.index;
    while p.index < p.source.len() && p.source.as_bytes()[p.index].is_ascii_alphabetic() {
        p.index += 1;
    }
    if p.index > start {
        Some(&p.source[start..p.index])
    } else {
        None
    }
}

fn is_nth_start(p: &CssParser<'_>) -> bool {
    let r = p.remaining();
    if r.starts_with("even") || r.starts_with("odd") {
        return true;
    }
    let bytes = r.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    let b = bytes[0];
    b == b'+' || b == b'-' || b.is_ascii_digit()
}

fn read_nth<'a>(p: &mut CssParser<'a>) -> &'a str {
    let start = p.index;
    let bytes = p.source.as_bytes();
    while p.index < bytes.len() {
        let b = bytes[p.index];
        if b.is_ascii_alphanumeric()
            || b == b'+'
            || b == b'-'
            || b == b'n'
            || b.is_ascii_whitespace()
        {
            p.index += 1;
        } else {
            break;
        }
    }
    p.source[start..p.index].trim_end()
}

fn is_percentage_start(p: &CssParser<'_>) -> bool {
    let bytes = p.remaining().as_bytes();
    !bytes.is_empty() && bytes[0].is_ascii_digit()
}

fn read_percentage<'a>(p: &mut CssParser<'a>) -> &'a str {
    let start = p.index;
    let bytes = p.source.as_bytes();
    while p.index < bytes.len() && (bytes[p.index].is_ascii_digit() || bytes[p.index] == b'.') {
        p.index += 1;
    }
    if p.index < bytes.len() && bytes[p.index] == b'%' {
        p.index += 1;
    }
    &p.source[start..p.index]
}
