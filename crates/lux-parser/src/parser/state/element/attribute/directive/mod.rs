use lux_ast::common::Span;
use lux_ast::template::attribute::{AttributeNode, AttributeValue};
use winnow::Result;
use winnow::error::ContextError;

use crate::input::Input;

mod extract;
mod kinds;
mod modifier;

use kinds::{
    parse_animate, parse_bind, parse_class, parse_let, parse_on, parse_style, parse_transition,
    parse_use,
};

pub fn parse_directive<'a>(
    input: &mut Input<'a>,
    prefix: &str,
    name: &'a str,
    modifiers_str: Vec<&'a str>,
    value: Option<AttributeValue<'a>>,
    span: Span,
) -> Result<AttributeNode<'a>> {
    match prefix {
        "bind" => parse_bind(input, name, value, span),
        "on" => parse_on(name, value, modifiers_str, span),
        "class" => parse_class(input, name, value, span),
        "style" => parse_style(name, value, modifiers_str, span),
        "use" => parse_use(name, value, span),
        "let" => parse_let(name, value, span),
        "animate" => parse_animate(name, value, span),
        "in" => parse_transition(name, value, modifiers_str, span, true, false),
        "out" => parse_transition(name, value, modifiers_str, span, false, true),
        "transition" => parse_transition(name, value, modifiers_str, span, true, true),
        _ => Err(ContextError::new()),
    }
}
