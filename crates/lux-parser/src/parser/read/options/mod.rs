use lux_ast::template::attribute::{Attribute, AttributeNode};
use lux_ast::template::element::SvelteOptionsRaw;
use lux_ast::template::root::{CssOption, Namespace, SvelteOptions};
use oxc_allocator::Allocator;

use crate::error::{ErrorKind, ParseError};

mod custom_element;
mod static_value;

use custom_element::parse_custom_element_options;
use static_value::{get_boolean_value, get_static_string_value};

pub fn process_svelte_options<'a>(
    raw: SvelteOptionsRaw<'a>,
    allocator: &'a Allocator,
) -> Result<SvelteOptions<'a>, ParseError> {
    let span = raw.span;

    if !raw.fragment.nodes.is_empty() {
        return Err(ParseError::with_code(
            ErrorKind::InvalidSvelteOptions,
            "svelte_meta_invalid_content",
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
                return Err(ParseError::with_code(
                    ErrorKind::InvalidSvelteOptions,
                    "svelte_options_invalid_attribute",
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
                return Err(ParseError::with_code(
                    ErrorKind::InvalidSvelteOptions,
                    "svelte_options_deprecated_tag",
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
                        return Err(ParseError::with_code(
                            ErrorKind::InvalidSvelteOptions,
                            "svelte_options_invalid_attribute_value",
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
                    return Err(ParseError::with_code(
                        ErrorKind::InvalidSvelteOptions,
                        "svelte_options_invalid_attribute_value",
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
                options.custom_element = parse_custom_element_options(attr, allocator)?;
            }
            _ => {
                return Err(ParseError::with_code(
                    ErrorKind::InvalidSvelteOptions,
                    "svelte_options_unknown_attribute",
                    attr.span,
                    format!("Unknown svelte:options attribute: {}", attr.name),
                ));
            }
        }
    }

    options.attributes = attributes;
    Ok(options)
}
