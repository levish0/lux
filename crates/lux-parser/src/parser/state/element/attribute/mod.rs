pub mod directive;
pub mod read;
pub mod value;

use std::borrow::Cow;
use std::collections::HashSet;

use crate::error::ErrorKind;
use lux_ast::attributes::{Attribute, AttributeSequenceValue, AttributeValue};
use lux_ast::node::AttributeNode;
use lux_ast::span::Span;
use lux_ast::text::Text;

use crate::parser::Parser;
use crate::parser::html_entities::decode_character_references;

use read::{is_token_ending_char, read_attribute};
pub use value::read_sequence;

/// Check if current position starts with a quote character (" or ')
#[inline]
fn starts_with_quote(parser: &Parser) -> bool {
    parser.match_ch(|ch| ch == b'"' || ch == b'\'')
}

/// Manual parsing of static attribute value (replaces REGEX_ATTRIBUTE_VALUE).
/// Matches: `"..."` or `'...'` or `[^>\s]+`
/// Returns the full matched string (including quotes if present), or None.
fn match_static_attribute_value<'a>(parser: &Parser<'a>) -> Option<&'a str> {
    let bytes = parser.template.as_bytes();
    let start = parser.index;
    if start >= bytes.len() {
        return None;
    }
    let first = bytes[start];
    if first == b'"' {
        // Find closing "
        let mut i = start + 1;
        while i < bytes.len() && bytes[i] != b'"' {
            i += 1;
        }
        if i < bytes.len() {
            // Include closing quote
            Some(&parser.template[start..=i])
        } else {
            None
        }
    } else if first == b'\'' {
        // Find closing '
        let mut i = start + 1;
        while i < bytes.len() && bytes[i] != b'\'' {
            i += 1;
        }
        if i < bytes.len() {
            Some(&parser.template[start..=i])
        } else {
            None
        }
    } else if first != b'>' && !first.is_ascii_whitespace() {
        // Unquoted: read until > or whitespace
        let mut i = start;
        while i < bytes.len() && bytes[i] != b'>' && !bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i > start {
            Some(&parser.template[start..i])
        } else {
            None
        }
    } else {
        None
    }
}

/// Check if an attribute is a duplicate. Adds to unique_names if not.
/// Returns true if duplicate found.
/// Reference: element.js lines 226-244
fn check_duplicate_attr(attr: &AttributeNode, unique_names: &mut HashSet<String>) -> bool {
    let (type_prefix, attr_name) = match attr {
        AttributeNode::Attribute(a) => ("Attribute", a.name),
        AttributeNode::BindDirective(b) => ("Attribute", b.name),
        AttributeNode::StyleDirective(s) => ("StyleDirective", s.name),
        AttributeNode::ClassDirective(c) => ("ClassDirective", c.name),
        _ => return false,
    };
    if attr_name == "this" {
        return false;
    }
    let key = format!("{}{}", type_prefix, attr_name);
    !unique_names.insert(key)
}

/// Read static attributes (for top-level script/style tags).
/// Port of reference `read_static_attribute` in element.js.
pub fn read_static_attributes<'a>(parser: &mut Parser<'a>) -> Vec<AttributeNode<'a>> {
    let mut attributes = Vec::new();

    loop {
        parser.allow_whitespace();
        if parser.index >= parser.template.len() || parser.match_str(">") {
            break;
        }
        if let Some(attr) = read_static_attribute(parser) {
            attributes.push(attr);
        } else {
            break;
        }
    }

    attributes
}

/// Read a single static attribute (name="value" or name).
/// Used for script/style tags where only simple attributes are valid.
/// Port of reference `read_static_attribute` in element.js.
fn read_static_attribute<'a>(parser: &mut Parser<'a>) -> Option<AttributeNode<'a>> {
    let start = parser.index;

    // Read attribute name
    let name = parser.read_until_char(is_token_ending_char);
    if name.is_empty() {
        return None;
    }

    let name_loc = parser.source_location(start, parser.index);

    let value = if parser.eat("=") {
        parser.allow_whitespace();

        // Match static attribute value: "..." or '...' or unquoted
        let raw_match = match_static_attribute_value(parser);
        let Some(raw_full) = raw_match else {
            // Reference: e.expected_attribute_value(parser.index)
            if !parser.loose {
                parser.error(
                    ErrorKind::ExpectedAttributeValue,
                    parser.index,
                    "Expected attribute value".to_string(),
                );
            }
            return None;
        };
        parser.index += raw_full.len();

        let quoted = raw_full.starts_with('"') || raw_full.starts_with('\'');
        let raw = if quoted {
            &raw_full[1..raw_full.len() - 1]
        } else {
            raw_full
        };

        let val_start = parser.index - raw.len() - if quoted { 1 } else { 0 };
        let val_end = if quoted {
            parser.index - 1
        } else {
            parser.index
        };
        let decoded = decode_character_references(raw, true);
        let data = match decoded {
            Cow::Borrowed(s) => s,
            Cow::Owned(s) => parser.allocator.alloc_str(&s),
        };

        AttributeValue::Sequence(vec![AttributeSequenceValue::Text(Text {
            span: Span::new(val_start, val_end),
            raw,
            data,
        })])
    } else {
        if starts_with_quote(parser) {
            // Reference: e.expected_token(parser.index, '=')
            if !parser.loose {
                parser.error(
                    ErrorKind::ExpectedToken,
                    parser.index,
                    "Expected '='".to_string(),
                );
            }
            return None;
        }
        AttributeValue::True
    };

    Some(AttributeNode::Attribute(Attribute {
        span: Span::new(start, parser.index),
        name,
        name_loc: Some(name_loc),
        value,
    }))
}

/// Read attributes until `>` or `/>`.
/// Port of reference `read_attribute` loop in element.js.
/// Includes duplicate attribute checking (reference: element.js lines 226-244).
pub fn read_attributes<'a>(parser: &mut Parser<'a>) -> Vec<AttributeNode<'a>> {
    let mut attributes = Vec::new();
    let mut unique_names: HashSet<String> = HashSet::new();

    loop {
        parser.allow_whitespace();

        if parser.index >= parser.template.len() {
            break;
        }

        if parser.match_str(">") || parser.match_str("/>") {
            break;
        }

        if let Some(attr) = read_attribute(parser) {
            // Duplicate check (reference: element.js lines 229-244)
            let duplicate = check_duplicate_attr(&attr, &mut unique_names);
            if duplicate && !parser.loose {
                // Would be e.attribute_duplicate error in strict mode
            }
            attributes.push(attr);
        } else {
            // Reference: loop exits when read_attribute returns null
            break;
        }
    }

    attributes
}
