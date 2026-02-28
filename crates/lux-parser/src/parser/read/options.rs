use lux_ast::template::attribute::{Attribute, AttributeNode, AttributeValue};
use lux_ast::template::element::SvelteOptionsRaw;
use lux_ast::template::root::{CssOption, CustomElementOptions, Namespace, SvelteOptions};
use lux_ast::template::tag::TextOrExpressionTag;
use oxc_ast::ast::Expression;

use crate::error::{ErrorKind, ParseError};

pub fn process_svelte_options<'a>(
    raw: SvelteOptionsRaw<'a>,
) -> Result<SvelteOptions<'a>, ParseError> {
    let span = raw.span;

    if !raw.fragment.nodes.is_empty() {
        return Err(ParseError::new(
            ErrorKind::InvalidSvelteOptions,
            span,
            "svelte:options cannot have child nodes",
        ));
    }

    // Extract Attribute nodes, reject non-Attribute nodes
    let mut attributes: Vec<Attribute<'a>> = Vec::new();
    for attr_node in raw.attributes {
        match attr_node {
            AttributeNode::Attribute(a) => attributes.push(a),
            _ => {
                return Err(ParseError::new(
                    ErrorKind::InvalidSvelteOptions,
                    span,
                    "svelte:options can only have static attributes",
                ));
            }
        }
    }

    let mut options = SvelteOptions {
        span,
        runes: None,
        immutable: None,
        accessors: None,
        preserve_whitespace: None,
        namespace: None,
        css: None,
        custom_element: None,
        attributes: Vec::new(),
    };

    for attr in &attributes {
        match attr.name {
            "runes" => {
                options.runes = Some(get_boolean_value(attr)?);
            }
            "tag" => {
                return Err(ParseError::new(
                    ErrorKind::InvalidSvelteOptions,
                    attr.span,
                    "svelte:options tag is deprecated, use customElement instead",
                ));
            }
            "namespace" => {
                let value = get_static_string_value(attr)?;
                options.namespace = Some(match value {
                    "html" => Namespace::Html,
                    "svg" | "http://www.w3.org/2000/svg" => Namespace::Svg,
                    "mathml" | "http://www.w3.org/1998/Math/MathML" => Namespace::Mathml,
                    _ => {
                        return Err(ParseError::new(
                            ErrorKind::InvalidSvelteOptions,
                            attr.span,
                            "namespace must be \"html\", \"mathml\" or \"svg\"",
                        ));
                    }
                });
            }
            "css" => {
                let value = get_static_string_value(attr)?;
                if value == "injected" {
                    options.css = Some(CssOption::Injected);
                } else {
                    return Err(ParseError::new(
                        ErrorKind::InvalidSvelteOptions,
                        attr.span,
                        "css must be \"injected\"",
                    ));
                }
            }
            "immutable" => {
                options.immutable = Some(get_boolean_value(attr)?);
            }
            "preserveWhitespace" => {
                options.preserve_whitespace = Some(get_boolean_value(attr)?);
            }
            "accessors" => {
                options.accessors = Some(get_boolean_value(attr)?);
            }
            "customElement" => {
                let tag = get_static_string_value(attr)?;
                options.custom_element = Some(CustomElementOptions {
                    tag: Some(tag),
                    shadow: None,
                    props: None,
                    extend: None,
                });
            }
            _ => {
                return Err(ParseError::new(
                    ErrorKind::InvalidSvelteOptions,
                    attr.span,
                    format!("Unknown svelte:options attribute: {}", attr.name),
                ));
            }
        }
    }

    options.attributes = attributes;
    Ok(options)
}

enum StaticValue<'a> {
    String(&'a str),
    Bool(bool),
}

fn get_static_value<'a>(attr: &Attribute<'a>) -> Result<StaticValue<'a>, ParseError> {
    match &attr.value {
        AttributeValue::True => Ok(StaticValue::Bool(true)),
        AttributeValue::ExpressionTag(tag) => expression_to_static_value(&tag.expression, attr),
        AttributeValue::Sequence(chunks) if chunks.len() == 1 => match &chunks[0] {
            TextOrExpressionTag::Text(t) => Ok(StaticValue::String(t.data)),
            TextOrExpressionTag::ExpressionTag(tag) => {
                expression_to_static_value(&tag.expression, attr)
            }
        },
        _ => Err(ParseError::new(
            ErrorKind::InvalidSvelteOptions,
            attr.span,
            format!("{} must be a static value", attr.name),
        )),
    }
}

fn expression_to_static_value<'a>(
    expression: &Expression<'a>,
    attr: &Attribute<'a>,
) -> Result<StaticValue<'a>, ParseError> {
    match expression {
        Expression::StringLiteral(string) => Ok(StaticValue::String(string.value.as_str())),
        Expression::BooleanLiteral(boolean) => Ok(StaticValue::Bool(boolean.value)),
        _ => Err(ParseError::new(
            ErrorKind::InvalidSvelteOptions,
            attr.span,
            format!("{} must be a static value", attr.name),
        )),
    }
}

fn get_static_string_value<'a>(attr: &Attribute<'a>) -> Result<&'a str, ParseError> {
    match get_static_value(attr)? {
        StaticValue::String(value) => Ok(value),
        StaticValue::Bool(_) => Err(ParseError::new(
            ErrorKind::InvalidSvelteOptions,
            attr.span,
            format!("{} must be a string", attr.name),
        )),
    }
}

fn get_boolean_value(attr: &Attribute<'_>) -> Result<bool, ParseError> {
    match get_static_value(attr)? {
        StaticValue::Bool(value) => Ok(value),
        StaticValue::String("true") => Ok(true),
        StaticValue::String("false") => Ok(false),
        _ => Err(ParseError::new(
            ErrorKind::InvalidSvelteOptions,
            attr.span,
            format!("{} must be true or false", attr.name),
        )),
    }
}
