use lux_ast::template::attribute::Attribute;

use crate::error::{ErrorKind, ParseError};

use super::super::static_value::{StaticValue, get_static_value};
use super::errors::invalid_custom_element_tag_name;

const RESERVED_CUSTOM_ELEMENT_TAG_NAMES: &[&str] = &[
    "annotation-xml",
    "color-profile",
    "font-face",
    "font-face-src",
    "font-face-uri",
    "font-face-format",
    "font-face-name",
    "missing-glyph",
];

pub(super) fn get_custom_element_tag<'a>(attr: &Attribute<'a>) -> Result<&'a str, ParseError> {
    match get_static_value(attr) {
        Ok(StaticValue::String(value)) => Ok(value),
        _ => Err(invalid_custom_element_tag_name(attr)),
    }
}

pub(super) fn validate_custom_element_tag_name(
    attr: &Attribute<'_>,
    tag: &str,
) -> Result<(), ParseError> {
    if !is_valid_custom_element_name(tag) {
        return Err(invalid_custom_element_tag_name(attr));
    }

    if RESERVED_CUSTOM_ELEMENT_TAG_NAMES.contains(&tag) {
        return Err(ParseError::with_code(
            ErrorKind::InvalidSvelteOptions,
            "svelte_options_reserved_tagname",
            attr.span,
            "Tag name is reserved",
        ));
    }

    Ok(())
}

fn is_valid_custom_element_name(tag: &str) -> bool {
    let mut chars = tag.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !first.is_ascii_lowercase() {
        return false;
    }

    let mut has_hyphen = false;

    for ch in chars {
        if ch == '-' {
            has_hyphen = true;
            continue;
        }

        if ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '.' {
            continue;
        }

        if ch.is_ascii() {
            return false;
        }

        if ch.is_whitespace()
            || ch.is_control()
            || matches!(ch, '<' | '>' | '"' | '\'' | '=' | '/' | '\\')
        {
            return false;
        }
    }

    has_hyphen
}
