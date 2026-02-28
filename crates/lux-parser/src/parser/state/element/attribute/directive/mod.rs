use lux_ast::common::Span;
use lux_ast::template::attribute::{AttributeNode, AttributeValue};
use lux_ast::template::directive::{
    AnimateDirective, BindDirective, ClassDirective, LetDirective, OnDirective, StyleDirective,
    StyleDirectiveValue, TransitionDirective, UseDirective,
};
use winnow::Result;
use winnow::error::ContextError;

use crate::input::Input;

mod extract;
mod modifier;

use extract::{extract_directive_expression, extract_expression_from_value};
use modifier::{parse_event_modifier, parse_style_modifier, parse_transition_modifier};

pub fn parse_directive<'a>(
    input: &mut Input<'a>,
    prefix: &str,
    name: &'a str,
    modifiers_str: Vec<&'a str>,
    value: Option<AttributeValue<'a>>,
    span: Span,
) -> Result<AttributeNode<'a>> {
    match prefix {
        "bind" => {
            let expression = extract_directive_expression(input, value, name)?;
            Ok(AttributeNode::BindDirective(BindDirective {
                span,
                name,
                expression,
            }))
        }
        "on" => {
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
        "class" => {
            let expression = extract_directive_expression(input, value, name)?;
            Ok(AttributeNode::ClassDirective(ClassDirective {
                span,
                name,
                expression,
            }))
        }
        "style" => {
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
        "use" => {
            let expression = value.map(extract_expression_from_value).transpose()?;
            Ok(AttributeNode::UseDirective(UseDirective {
                span,
                name,
                expression,
            }))
        }
        "let" => {
            let expression = value.map(extract_expression_from_value).transpose()?;
            Ok(AttributeNode::LetDirective(LetDirective {
                span,
                name,
                expression,
            }))
        }
        "animate" => {
            let expression = value.map(extract_expression_from_value).transpose()?;
            Ok(AttributeNode::AnimateDirective(AnimateDirective {
                span,
                name,
                expression,
            }))
        }
        "in" => build_transition(name, value, modifiers_str, span, true, false),
        "out" => build_transition(name, value, modifiers_str, span, false, true),
        "transition" => build_transition(name, value, modifiers_str, span, true, true),
        _ => Err(ContextError::new()),
    }
}

fn build_transition<'a>(
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
