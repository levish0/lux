mod bind_class;
mod event;
mod simple;
mod style;
mod transition;

use lux_ast::common::Span;
use lux_ast::template::attribute::{AttributeNode, AttributeValue};
use winnow::Result;

use crate::input::Input;

pub(super) fn parse_bind<'a>(
    input: &mut Input<'a>,
    name: &'a str,
    value: Option<AttributeValue<'a>>,
    span: Span,
) -> Result<AttributeNode<'a>> {
    bind_class::parse_bind(input, name, value, span)
}

pub(super) fn parse_class<'a>(
    input: &mut Input<'a>,
    name: &'a str,
    value: Option<AttributeValue<'a>>,
    span: Span,
) -> Result<AttributeNode<'a>> {
    bind_class::parse_class(input, name, value, span)
}

pub(super) fn parse_on<'a>(
    name: &'a str,
    value: Option<AttributeValue<'a>>,
    modifiers_str: Vec<&'a str>,
    span: Span,
) -> Result<AttributeNode<'a>> {
    event::parse_on(name, value, modifiers_str, span)
}

pub(super) fn parse_style<'a>(
    name: &'a str,
    value: Option<AttributeValue<'a>>,
    modifiers_str: Vec<&'a str>,
    span: Span,
) -> Result<AttributeNode<'a>> {
    style::parse_style(name, value, modifiers_str, span)
}

pub(super) fn parse_use<'a>(
    name: &'a str,
    value: Option<AttributeValue<'a>>,
    span: Span,
) -> Result<AttributeNode<'a>> {
    simple::parse_use(name, value, span)
}

pub(super) fn parse_let<'a>(
    name: &'a str,
    value: Option<AttributeValue<'a>>,
    span: Span,
) -> Result<AttributeNode<'a>> {
    simple::parse_let(name, value, span)
}

pub(super) fn parse_animate<'a>(
    name: &'a str,
    value: Option<AttributeValue<'a>>,
    span: Span,
) -> Result<AttributeNode<'a>> {
    simple::parse_animate(name, value, span)
}

pub(super) fn parse_transition<'a>(
    name: &'a str,
    value: Option<AttributeValue<'a>>,
    modifiers_str: Vec<&'a str>,
    span: Span,
    intro: bool,
    outro: bool,
) -> Result<AttributeNode<'a>> {
    transition::parse_transition(name, value, modifiers_str, span, intro, outro)
}
