use lux_ast::template::attribute::{Attribute, AttributeNode, AttributeValue};
use lux_ast::template::element::SvelteOptionsRaw;
use lux_ast::template::root::{CssOption, CustomElementOptions, Namespace, SvelteOptions};
use lux_ast::template::tag::TextOrExpressionTag;

use crate::error::{ErrorKind, ParseError};

pub fn process_svelte_options<'a>(
    raw: SvelteOptionsRaw<'a>,
) -> Result<SvelteOptions<'a>, ParseError> {
    let span = raw.span;

    // Extract Attribute nodes, reject non-Attribute nodes
    let mut attributes: Vec<Attribute<'a>> = Vec::new();
    for attr_node in raw.attributes {
        match attr_node {
            AttributeNode::Attribute(a) => attributes.push(a),
            _ => {
                return Err(ParseError::new(
                    ErrorKind::General,
                    span,
                    "svelte:options can only have static attributes".to_string(),
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
                    ErrorKind::General,
                    attr.span,
                    "svelte:options tag is deprecated, use customElement instead".to_string(),
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
                            ErrorKind::General,
                            attr.span,
                            "namespace must be \"html\", \"mathml\" or \"svg\"".to_string(),
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
                        ErrorKind::General,
                        attr.span,
                        "css must be \"injected\"".to_string(),
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
                options.custom_element = Some(CustomElementOptions {
                    tag: get_static_string_value(attr).ok(),
                    shadow: None,
                    props: None,
                    extend: None,
                });
            }
            _ => {
                return Err(ParseError::new(
                    ErrorKind::General,
                    attr.span,
                    format!("Unknown svelte:options attribute: {}", attr.name),
                ));
            }
        }
    }

    options.attributes = attributes;
    Ok(options)
}

fn get_static_string_value<'a>(attr: &Attribute<'a>) -> Result<&'a str, ParseError> {
    match &attr.value {
        AttributeValue::Sequence(chunks) if chunks.len() == 1 => match &chunks[0] {
            TextOrExpressionTag::Text(t) => Ok(t.data),
            _ => Err(ParseError::new(
                ErrorKind::General,
                attr.span,
                format!("{} must be a static value", attr.name),
            )),
        },
        AttributeValue::True => Err(ParseError::new(
            ErrorKind::General,
            attr.span,
            format!("{} must have a value", attr.name),
        )),
        _ => Err(ParseError::new(
            ErrorKind::General,
            attr.span,
            format!("{} must be a static value", attr.name),
        )),
    }
}

fn get_boolean_value(attr: &Attribute<'_>) -> Result<bool, ParseError> {
    match &attr.value {
        AttributeValue::True => Ok(true),
        AttributeValue::Sequence(chunks) if chunks.len() == 1 => match &chunks[0] {
            TextOrExpressionTag::Text(t) if t.data == "true" => Ok(true),
            TextOrExpressionTag::Text(t) if t.data == "false" => Ok(false),
            _ => Err(ParseError::new(
                ErrorKind::General,
                attr.span,
                format!("{} must be true or false", attr.name),
            )),
        },
        _ => Err(ParseError::new(
            ErrorKind::General,
            attr.span,
            format!("{} must be true or false", attr.name),
        )),
    }
}
