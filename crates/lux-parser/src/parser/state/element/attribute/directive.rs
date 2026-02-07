use lux_ast::common::Span;
use lux_ast::template::attribute::{AttributeNode, AttributeValue};
use lux_ast::template::directive::{
    AnimateDirective, BindDirective, ClassDirective, EventModifier, LetDirective, OnDirective,
    StyleDirective, StyleDirectiveValue, StyleModifier, TransitionDirective, TransitionModifier,
    UseDirective,
};
use lux_ast::template::tag::TextOrExpressionTag;
use oxc_ast::ast::Expression;
use winnow::Result;
use winnow::error::ContextError;

use crate::input::Input;

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
                metadata: None,
            }))
        }
        "on" => {
            let expression = value.map(extract_expression_from_value).transpose()?;
            let modifiers = modifiers_str
                .iter()
                .filter_map(|m| parse_event_modifier(m))
                .collect();
            Ok(AttributeNode::OnDirective(OnDirective {
                span,
                name,
                expression,
                modifiers,
                metadata: None,
            }))
        }
        "class" => {
            let expression = extract_directive_expression(input, value, name)?;
            Ok(AttributeNode::ClassDirective(ClassDirective {
                span,
                name,
                expression,
                metadata: None,
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
                .filter_map(|m| match *m {
                    "important" => Some(StyleModifier::Important),
                    _ => None,
                })
                .collect();
            Ok(AttributeNode::StyleDirective(StyleDirective {
                span,
                name,
                value: style_value,
                modifiers,
                metadata: None,
            }))
        }
        "use" => {
            let expression = value.map(extract_expression_from_value).transpose()?;
            Ok(AttributeNode::UseDirective(UseDirective {
                span,
                name,
                expression,
                metadata: None,
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
                metadata: None,
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
        .filter_map(|m| parse_transition_modifier(m))
        .collect();
    Ok(AttributeNode::TransitionDirective(TransitionDirective {
        span,
        name,
        expression,
        modifiers,
        intro,
        outro,
        metadata: None,
    }))
}

fn extract_directive_expression<'a>(
    input: &mut Input<'a>,
    value: Option<AttributeValue<'a>>,
    name: &'a str,
) -> Result<Expression<'a>> {
    match value {
        Some(v) => extract_expression_from_value(v),
        None => {
            let allocator = input.state.allocator;
            let source_type = if input.state.ts {
                oxc_span::SourceType::ts()
            } else {
                oxc_span::SourceType::mjs()
            };
            oxc_parser::Parser::new(allocator, name, source_type)
                .parse_expression()
                .map_err(|_| ContextError::new())
        }
    }
}

fn extract_expression_from_value(value: AttributeValue<'_>) -> Result<Expression<'_>> {
    match value {
        AttributeValue::ExpressionTag(et) => Ok(et.expression),
        AttributeValue::Sequence(mut seq) => {
            if seq.len() == 1 {
                match seq.remove(0) {
                    TextOrExpressionTag::ExpressionTag(et) => Ok(et.expression),
                    _ => Err(ContextError::new()),
                }
            } else {
                Err(ContextError::new())
            }
        }
        AttributeValue::True => Err(ContextError::new()),
    }
}

fn parse_event_modifier(m: &str) -> Option<EventModifier> {
    match m {
        "capture" => Some(EventModifier::Capture),
        "nonpassive" => Some(EventModifier::Nonpassive),
        "once" => Some(EventModifier::Once),
        "passive" => Some(EventModifier::Passive),
        "preventDefault" => Some(EventModifier::PreventDefault),
        "self" => Some(EventModifier::Self_),
        "stopImmediatePropagation" => Some(EventModifier::StopImmediatePropagation),
        "stopPropagation" => Some(EventModifier::StopPropagation),
        "trusted" => Some(EventModifier::Trusted),
        _ => None,
    }
}

fn parse_transition_modifier(m: &str) -> Option<TransitionModifier> {
    match m {
        "local" => Some(TransitionModifier::Local),
        "global" => Some(TransitionModifier::Global),
        _ => None,
    }
}
