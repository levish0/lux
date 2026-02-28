use lux_ast::template::attribute::Attribute;
use lux_ast::template::root::{CssOption, Namespace, SvelteOptions};
use oxc_allocator::Allocator;

use crate::error::{ErrorKind, ParseError};

use super::custom_element::parse_custom_element_options;
use super::static_value::{get_boolean_value, get_static_string_value};

pub(super) fn apply_svelte_option<'a>(
    options: &mut SvelteOptions<'a>,
    attr: &Attribute<'a>,
    allocator: &'a Allocator,
) -> Result<(), ParseError> {
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
            options.namespace = Some(parse_namespace_option(attr)?);
        }
        "css" => {
            options.css = Some(parse_css_option(attr)?);
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

    Ok(())
}

fn parse_namespace_option(attr: &Attribute<'_>) -> Result<Namespace, ParseError> {
    let value = get_static_string_value(attr)?;
    match value {
        "html" => Ok(Namespace::Html),
        "svg" | "http://www.w3.org/2000/svg" => Ok(Namespace::Svg),
        "mathml" | "http://www.w3.org/1998/Math/MathML" => Ok(Namespace::Mathml),
        _ => Err(ParseError::with_code(
            ErrorKind::InvalidSvelteOptions,
            "svelte_options_invalid_attribute_value",
            attr.span,
            "namespace must be \"html\", \"mathml\" or \"svg\"",
        )),
    }
}

fn parse_css_option(attr: &Attribute<'_>) -> Result<CssOption, ParseError> {
    let value = get_static_string_value(attr)?;
    if value == "injected" {
        Ok(CssOption::Injected)
    } else {
        Err(ParseError::with_code(
            ErrorKind::InvalidSvelteOptions,
            "svelte_options_invalid_attribute_value",
            attr.span,
            "css must be \"injected\"",
        ))
    }
}
