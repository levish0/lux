use lux_ast::common::Span;
use lux_ast::template::attribute::{AttributeNode, AttributeValue};
use lux_ast::template::directive::{
    AnimateDirective, BindDirective, ClassDirective, LetDirective, OnDirective, StyleDirective,
    StyleDirectiveValue, TransitionDirective, UseDirective,
};
use winnow::Result;

use crate::input::Input;

use super::extract::{extract_directive_expression, extract_expression_from_value};
use super::modifier::{parse_event_modifier, parse_style_modifier, parse_transition_modifier};

pub(super) fn parse_bind<'a>(
    input: &mut Input<'a>,
    name: &'a str,
    value: Option<AttributeValue<'a>>,
    span: Span,
) -> Result<AttributeNode<'a>> {
    let expression = extract_directive_expression(input, value, name)?;
    Ok(AttributeNode::BindDirective(BindDirective {
        span,
        name,
        expression,
    }))
}

pub(super) fn parse_on<'a>(
    name: &'a str,
    value: Option<AttributeValue<'a>>,
    modifiers_str: Vec<&'a str>,
    span: Span,
) -> Result<AttributeNode<'a>> {
    let expression = value.map(extract_expression_from_value).transpose()?;
    let modifiers = modifiers_str
        .iter()
        .filter_map(|modifier| parse_event_modifier(modifier))
        .collect();

    Ok(AttributeNode::OnDirective(OnDirective {
        span,
        name,
        expression,
        modifiers,
    }))
}

pub(super) fn parse_class<'a>(
    input: &mut Input<'a>,
    name: &'a str,
    value: Option<AttributeValue<'a>>,
    span: Span,
) -> Result<AttributeNode<'a>> {
    let expression = extract_directive_expression(input, value, name)?;
    Ok(AttributeNode::ClassDirective(ClassDirective {
        span,
        name,
        expression,
    }))
}

pub(super) fn parse_style<'a>(
    name: &'a str,
    value: Option<AttributeValue<'a>>,
    modifiers_str: Vec<&'a str>,
    span: Span,
) -> Result<AttributeNode<'a>> {
    let style_value = match value {
        Some(AttributeValue::True) | None => StyleDirectiveValue::True,
        Some(AttributeValue::ExpressionTag(et)) => StyleDirectiveValue::ExpressionTag(et),
        Some(AttributeValue::Sequence(seq)) => StyleDirectiveValue::Sequence(seq),
    };

    let modifiers = modifiers_str
        .iter()
        .filter_map(|modifier| parse_style_modifier(modifier))
        .collect();

    Ok(AttributeNode::StyleDirective(StyleDirective {
        span,
        name,
        value: style_value,
        modifiers,
    }))
}

pub(super) fn parse_use<'a>(
    name: &'a str,
    value: Option<AttributeValue<'a>>,
    span: Span,
) -> Result<AttributeNode<'a>> {
    let expression = value.map(extract_expression_from_value).transpose()?;
    Ok(AttributeNode::UseDirective(UseDirective {
        span,
        name,
        expression,
    }))
}

pub(super) fn parse_let<'a>(
    name: &'a str,
    value: Option<AttributeValue<'a>>,
    span: Span,
) -> Result<AttributeNode<'a>> {
    let expression = value.map(extract_expression_from_value).transpose()?;
    Ok(AttributeNode::LetDirective(LetDirective {
        span,
        name,
        expression,
    }))
}

pub(super) fn parse_animate<'a>(
    name: &'a str,
    value: Option<AttributeValue<'a>>,
    span: Span,
) -> Result<AttributeNode<'a>> {
    let expression = value.map(extract_expression_from_value).transpose()?;
    Ok(AttributeNode::AnimateDirective(AnimateDirective {
        span,
        name,
        expression,
    }))
}

pub(super) fn parse_transition<'a>(
    name: &'a str,
    value: Option<AttributeValue<'a>>,
    modifiers_str: Vec<&'a str>,
    span: Span,
    intro: bool,
    outro: bool,
) -> Result<AttributeNode<'a>> {
    let expression = value.map(extract_expression_from_value).transpose()?;
    let modifiers = modifiers_str
        .iter()
        .filter_map(|modifier| parse_transition_modifier(modifier))
        .collect();

    Ok(AttributeNode::TransitionDirective(TransitionDirective {
        span,
        name,
        expression,
        modifiers,
        intro,
        outro,
    }))
}
