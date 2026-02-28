use lux_ast::template::attribute::Attribute;

use crate::error::{ErrorKind, ParseError};

pub(super) fn invalid_custom_element(attr: &Attribute<'_>) -> ParseError {
    ParseError::with_code(
        ErrorKind::InvalidSvelteOptions,
        "svelte_options_invalid_customelement",
        attr.span,
        "\"customElement\" must be a string literal tag name, null, or an object literal",
    )
}

pub(super) fn invalid_custom_element_props(attr: &Attribute<'_>) -> ParseError {
    ParseError::with_code(
        ErrorKind::InvalidSvelteOptions,
        "svelte_options_invalid_customelement_props",
        attr.span,
        "\"props\" must be an object literal with static keys and literal values",
    )
}

pub(super) fn invalid_custom_element_shadow(attr: &Attribute<'_>) -> ParseError {
    ParseError::with_code(
        ErrorKind::InvalidSvelteOptions,
        "svelte_options_invalid_customelement_shadow",
        attr.span,
        "\"shadow\" must be \"open\", \"none\", or an object literal",
    )
}

pub(super) fn invalid_custom_element_tag_name(attr: &Attribute<'_>) -> ParseError {
    ParseError::with_code(
        ErrorKind::InvalidSvelteOptions,
        "svelte_options_invalid_tagname",
        attr.span,
        "Tag name must be lowercase and hyphenated",
    )
}
