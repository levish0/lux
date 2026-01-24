use svelte_ast::attributes::{AttributeSequenceValue, AttributeValue};
use svelte_ast::metadata::ExpressionNodeMetadata;
use svelte_ast::span::Span;
use svelte_ast::tags::ExpressionTag;
use svelte_ast::text::Text;

use crate::error::ErrorKind;
use crate::parser::html_entities::decode_character_references;
use crate::parser::read::expression::read_expression;
use crate::parser::Parser;

/// Check if current position matches an invalid unquoted attribute value char.
/// Replaces regex_invalid_unquoted_attribute_value = /^(\/>|[\s"'=<>`])/
#[inline]
fn is_invalid_unquoted_attr_value(parser: &Parser) -> bool {
    let bytes = parser.template.as_bytes();
    let i = parser.index;
    if i >= bytes.len() {
        return false;
    }
    let ch = bytes[i];
    // Check for `/>`
    if ch == b'/' && bytes.get(i + 1).copied() == Some(b'>') {
        return true;
    }
    // Check for [\s"'=<>`]
    matches!(ch, b' ' | b'\t' | b'\r' | b'\n' | b'"' | b'\'' | b'=' | b'<' | b'>' | b'`')
}

/// Read an attribute value after `=`.
/// Port of reference `read_attribute_value` in element.js.
///
/// Returns:
/// - `ExpressionTag` for single `{expr}` without quotes
/// - `Sequence` for quoted values or multi-part values
pub fn read_attribute_value<'a>(parser: &mut Parser<'a>) -> AttributeValue<'a> {
    let quote_mark: Option<u8> = if parser.eat("'") {
        Some(b'\'')
    } else if parser.eat("\"") {
        Some(b'"')
    } else {
        None
    };

    // Empty quoted value: "" or ''
    if let Some(q) = quote_mark {
        if parser.index < parser.template.len() && parser.template.as_bytes()[parser.index] == q {
            parser.index += 1; // consume closing quote
            let pos = parser.index - 1;
            return AttributeValue::Sequence(vec![AttributeSequenceValue::Text(Text {
                span: Span::new(pos, pos),
                raw: String::new(),
                data: String::new(),
            })]);
        }
    }

    // Read sequence until done condition
    let chunks = if let Some(q) = quote_mark {
        read_sequence(parser, move |p| {
            p.index < p.template.len() && p.template.as_bytes()[p.index] == q
        })
    } else {
        // Unquoted: stop at regex_invalid_unquoted_attribute_value
        read_sequence(parser, |p| {
            if p.index >= p.template.len() {
                return true;
            }
            is_invalid_unquoted_attr_value(p)
        })
    };

    // Consume closing quote
    if quote_mark.is_some() {
        parser.index += 1;
    }

    // Reference: if (value.length === 0 && !quote_mark) e.expected_attribute_value(parser.index)
    if chunks.is_empty() && quote_mark.is_none() {
        if !parser.loose {
            parser.error(
                ErrorKind::ExpectedAttributeValue,
                parser.index,
                "Expected attribute value".to_string(),
            );
        }
        return AttributeValue::True;
    }

    if chunks.is_empty() {
        return AttributeValue::True;
    }

    // Reference logic for return type:
    // if (quote_mark || value.length > 1 || value[0].type === 'Text') → return array (Sequence)
    // else → return value[0] (single ExpressionTag)
    if quote_mark.is_some() || chunks.len() > 1 {
        return AttributeValue::Sequence(chunks);
    }

    // Single chunk, no quotes
    let chunk = chunks.into_iter().next().unwrap();
    match chunk {
        AttributeSequenceValue::Text(t) => {
            AttributeValue::Sequence(vec![AttributeSequenceValue::Text(t)])
        }
        AttributeSequenceValue::ExpressionTag(et) => AttributeValue::ExpressionTag(et),
    }
}

/// Byte predicate: matches /[^a-z]/ — non-lowercase-alpha
#[inline]
fn is_non_alpha(ch: u8) -> bool {
    !(b'a'..=b'z').contains(&ch)
}

/// Read a sequence of Text and ExpressionTag chunks.
/// Port of reference `read_sequence` in element.js.
///
/// `done` is a closure that returns true when reading should stop.
pub fn read_sequence<'a>(
    parser: &mut Parser<'a>,
    done: impl Fn(&Parser<'a>) -> bool,
) -> Vec<AttributeSequenceValue<'a>> {
    let mut chunks: Vec<AttributeSequenceValue<'a>> = Vec::new();
    let mut text_start = parser.index;
    let mut raw = String::new();

    while parser.index < parser.template.len() {
        if done(parser) {
            // Flush any pending text
            if !raw.is_empty() {
                let data = decode_character_references(&raw, true);
                chunks.push(AttributeSequenceValue::Text(Text {
                    span: Span::new(text_start, parser.index),
                    raw: raw.clone(),
                    data,
                }));
            }
            return chunks;
        }

        if parser.eat("{") {
            // Reference: check for {#block} and {@tag} invalid placement
            if parser.match_str("#") {
                let block_start = parser.index - 1;
                parser.index += 1; // skip '#'
                let block_name = parser.read_until_char(is_non_alpha).to_string();
                if !parser.loose {
                    parser.error(
                        ErrorKind::BlockInvalidPlacement,
                        block_start,
                        format!("`{{#{block_name}}}` block cannot be used in attribute value"),
                    );
                }
            } else if parser.match_str("@") {
                let tag_start = parser.index - 1;
                parser.index += 1; // skip '@'
                let tag_name = parser.read_until_char(is_non_alpha).to_string();
                if !parser.loose {
                    parser.error(
                        ErrorKind::TagInvalidPlacement,
                        tag_start,
                        format!("`{{@{tag_name}}}` tag cannot be used in attribute value"),
                    );
                }
            }

            // Flush pending text
            if !raw.is_empty() {
                let data = decode_character_references(&raw, true);
                chunks.push(AttributeSequenceValue::Text(Text {
                    span: Span::new(text_start, parser.index - 1),
                    raw: raw.clone(),
                    data,
                }));
                raw.clear();
            }

            let expr_start = parser.index - 1; // include the `{`
            parser.allow_whitespace();
            let expression = match read_expression(parser) {
                Ok(expr) => expr,
                Err(_) => {
                    skip_to_closing_brace_attr(parser);
                    text_start = parser.index;
                    continue;
                }
            };
            parser.allow_whitespace();
            parser.eat_required("}").ok();

            chunks.push(AttributeSequenceValue::ExpressionTag(ExpressionTag {
                span: Span::new(expr_start, parser.index),
                expression,
                metadata: ExpressionNodeMetadata::default(),
            }));

            text_start = parser.index;
        } else {
            // Fix: properly handle UTF-8 characters instead of byte-by-byte
            let remaining = &parser.template[parser.index..];
            if let Some(ch) = remaining.chars().next() {
                raw.push(ch);
                parser.index += ch.len_utf8();
            } else {
                parser.index += 1;
            }
        }
    }

    // Reference: e.unexpected_eof in non-loose mode
    if parser.loose {
        // Flush remaining text
        if !raw.is_empty() {
            let data = decode_character_references(&raw, true);
            chunks.push(AttributeSequenceValue::Text(Text {
                span: Span::new(text_start, parser.index),
                raw,
                data,
            }));
        }
        chunks
    } else {
        // Flush remaining text before erroring
        if !raw.is_empty() {
            let data = decode_character_references(&raw, true);
            chunks.push(AttributeSequenceValue::Text(Text {
                span: Span::new(text_start, parser.index),
                raw,
                data,
            }));
        }
        parser.error(
            ErrorKind::UnexpectedEof,
            parser.template.len(),
            "Unexpected end of input".to_string(),
        );
        chunks
    }
}

/// Skip to closing `}` for attribute expressions.
pub fn skip_to_closing_brace_attr(parser: &mut Parser) {
    let mut depth = 1u32;
    while parser.index < parser.template.len() && depth > 0 {
        let ch = parser.template.as_bytes()[parser.index];
        match ch {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    parser.index += 1;
                    return;
                }
            }
            b'\'' | b'"' | b'`' => {
                skip_string(parser, ch);
                continue;
            }
            _ => {}
        }
        parser.index += 1;
    }
}

/// Skip a string literal (single, double, or template).
fn skip_string(parser: &mut Parser, quote: u8) {
    parser.index += 1;
    while parser.index < parser.template.len() {
        let ch = parser.template.as_bytes()[parser.index];
        if ch == b'\\' {
            parser.index += 1;
        } else if ch == quote {
            return;
        } else if quote == b'`' && ch == b'$' {
            if parser.index + 1 < parser.template.len()
                && parser.template.as_bytes()[parser.index + 1] == b'{'
            {
                parser.index += 2;
                let mut depth = 1u32;
                while parser.index < parser.template.len() && depth > 0 {
                    let c = parser.template.as_bytes()[parser.index];
                    match c {
                        b'{' => depth += 1,
                        b'}' => depth -= 1,
                        _ => {}
                    }
                    if depth > 0 {
                        parser.index += 1;
                    }
                }
                if depth == 0 {
                    parser.index += 1;
                }
                continue;
            }
        }
        parser.index += 1;
    }
}
