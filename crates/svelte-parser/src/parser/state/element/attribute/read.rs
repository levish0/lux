use svelte_ast::attributes::{Attribute, AttributeSequenceValue, AttributeValue, SpreadAttribute};
use svelte_ast::metadata::ExpressionNodeMetadata;
use svelte_ast::node::AttributeNode;
use svelte_ast::span::Span;
use svelte_ast::tags::{AttachTag, ExpressionTag};
use svelte_ast::text::Text;

use crate::error::ErrorKind;
use crate::parser::read::expression::read_expression;
use crate::parser::Parser;

use super::directive::{build_directive, get_directive_type, make_identifier};
use super::value::{read_attribute_value, skip_to_closing_brace_attr};

/// Byte predicate: matches /[\s=/>"']/ — token ending characters
#[inline]
pub fn is_token_ending_char(ch: u8) -> bool {
    matches!(ch, b' ' | b'\t' | b'\r' | b'\n' | b'=' | b'/' | b'>' | b'"' | b'\'')
}

/// Check if current position starts with a quote character (" or ')
#[inline]
fn starts_with_quote(parser: &Parser) -> bool {
    parser.match_ch(|ch| ch == b'"' || ch == b'\'')
}

/// Read a single attribute (or spread, or shorthand).
/// Port of reference `read_attribute` in element.js.
pub fn read_attribute<'a>(parser: &mut Parser<'a>) -> Option<AttributeNode<'a>> {
    let start = parser.index;

    // Handle `{...}` — attach, spread, or shorthand
    if parser.eat("{") {
        parser.allow_whitespace();

        // {@attach expr}
        if parser.eat("@attach") {
            if let Err(_) = parser.require_whitespace() {
                if !parser.loose {
                    return None;
                }
            }
            let expression = match read_expression(parser) {
                Ok(expr) => expr,
                Err(_) => {
                    skip_to_closing_brace_attr(parser);
                    return None;
                }
            };
            parser.allow_whitespace();
            parser.eat_required("}").ok();

            return Some(AttributeNode::AttachTag(AttachTag {
                span: Span::new(start, parser.index),
                expression,
            }));
        }

        // Spread attribute: {...expr}
        if parser.eat("...") {
            let expression = match read_expression(parser) {
                Ok(expr) => expr,
                Err(_) => {
                    skip_to_closing_brace_attr(parser);
                    return None;
                }
            };
            parser.allow_whitespace();
            parser.eat_required("}").ok();

            return Some(AttributeNode::SpreadAttribute(SpreadAttribute {
                span: Span::new(start, parser.index),
                expression,
            }));
        }

        // Shorthand: {name}
        let (id_name, id_start, id_end) = parser.read_identifier();
        if id_name.is_empty() {
            // Reference: element.js lines 551-562
            if parser.loose
                && (parser.match_str("#")
                    || parser.match_str("/")
                    || parser.match_str("@")
                    || parser.match_str(":"))
            {
                // In an unclosed opening tag, likely part of a block
                parser.index = start;
                return None;
            } else if parser.loose && parser.match_str("}") {
                // Likely in the middle of typing, just created the shorthand — allow
                parser.eat_required("}").ok();
                return None;
            } else {
                // Reference: e.attribute_empty_shorthand(start)
                if !parser.loose {
                    parser.error(
                        ErrorKind::AttributeEmptyShorthand,
                        start,
                        "Attribute shorthand cannot be empty".to_string(),
                    );
                }
                skip_to_closing_brace_attr(parser);
                return None;
            }
        }
        let id_name_str = id_name.to_string();

        parser.allow_whitespace();
        parser.eat_required("}").ok();

        // Create identifier expression for the shorthand
        let expression = make_identifier(parser, id_name, id_start, id_end);

        let expr_tag = ExpressionTag {
            span: Span::new(id_start, id_end),
            expression,
            metadata: ExpressionNodeMetadata::default(),
        };

        let name_loc = parser.source_location(id_start, id_end);

        return Some(AttributeNode::Attribute(Attribute {
            span: Span::new(start, parser.index),
            name: id_name_str,
            name_loc: Some(name_loc),
            value: AttributeValue::ExpressionTag(expr_tag),
        }));
    }

    // Read attribute name — consume until whitespace, =, /, >, ", '
    let name_start = parser.index;
    let name = read_tag(parser);
    let name_end = parser.index;

    if name.is_empty() {
        return None;
    }

    let name_loc = parser.source_location(name_start, name_end);

    let mut end = parser.index;

    parser.allow_whitespace();

    // Check for directive type before reading value
    let colon_idx = name.find(':');
    let is_directive = colon_idx
        .map(|i| get_directive_type(&name[..i]))
        .unwrap_or(false);

    // Read value
    let value = if parser.eat("=") {
        parser.allow_whitespace();

        // Edge case: value=/>  (the '/' is the value, not self-closing)
        if parser.match_str("/")
            && parser.template.get(parser.index + 1..parser.index + 2) == Some(">")
        {
            let char_start = parser.index;
            parser.index += 1; // consume '/'
            end = parser.index;
            AttributeValue::Sequence(vec![AttributeSequenceValue::Text(Text {
                span: Span::new(char_start, char_start + 1),
                raw: "/".to_string(),
                data: "/".to_string(),
            })])
        } else {
            let v = read_attribute_value(parser);
            end = parser.index;
            v
        }
    } else if starts_with_quote(parser) {
        // Quote without '=' — error in strict mode (reference: e.expected_token)
        if !parser.loose {
            return None; // Would be an error, skip attribute
        }
        AttributeValue::True
    } else {
        AttributeValue::True
    };

    // Directive handling (name contains ':' with valid prefix)
    if let Some(colon_idx) = colon_idx {
        if is_directive {
            let prefix = &name[..colon_idx];
            let directive_name = &name[colon_idx + 1..];

            // Split modifiers by '|'
            let parts: Vec<&str> = directive_name.split('|').collect();
            let dir_name = parts[0].to_string();
            let modifiers: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

            return build_directive(
                parser, prefix, &dir_name, &modifiers, value, name_loc, start, end,
            );
        }
    }

    Some(AttributeNode::Attribute(Attribute {
        span: Span::new(start, end),
        name,
        name_loc: Some(name_loc),
        value,
    }))
}

/// Read an attribute/tag name: consume until token ending character.
/// Port of reference `read_tag(parser, regex_token_ending_character)`.
fn read_tag(parser: &mut Parser) -> String {
    parser.read_until_char(is_token_ending_char).to_string()
}
