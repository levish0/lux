mod rules;
mod selectors;

use svelte_ast::css::{CssContent, StyleSheet};
use svelte_ast::node::StyleSheetChild;
use svelte_ast::span::Span;
use winnow::Result as ParseResult;
use winnow::combinator::peek;
use winnow::prelude::*;
use winnow::stream::Location;
use winnow::token::{literal, take_until, take_while};

use super::ParserInput;
use super::attribute::attribute_parser;
use svelte_ast::node::AttributeNode;

use self::rules::css_child_parser;

/// Parse a `<style>` tag and return a StyleSheet node.
/// Consumes from `<style` to `</style>`.
pub fn style_parser(parser_input: &mut ParserInput) -> ParseResult<StyleSheet> {
    let start = parser_input.current_token_start();

    // Consume <style
    literal("<style").parse_next(parser_input)?;

    // Parse attributes
    let attributes = parse_style_attributes(parser_input)?;

    // Consume >
    literal(">").parse_next(parser_input)?;

    let content_start = parser_input.current_token_start() as u32;

    // Read raw content until </style> (zero-allocation slice)
    let styles: &str = take_until(0.., "</style>").parse_next(parser_input)?;
    let styles = styles.to_string();
    let content_end = parser_input.current_token_start() as u32;

    // Consume </style>
    literal("</style>").parse_next(parser_input)?;
    let end = parser_input.previous_token_end();

    // Parse the CSS content
    let children = parse_stylesheet_content(&styles, content_start)?;

    Ok(StyleSheet {
        span: Span::new(start, end),
        attributes: attributes
            .into_iter()
            .filter_map(|a| match a {
                AttributeNode::Attribute(attr) => Some(attr),
                _ => None,
            })
            .collect(),
        children,
        content: CssContent {
            start: content_start,
            end: content_end,
            styles,
            comment: None,
        },
    })
}

fn parse_style_attributes(parser_input: &mut ParserInput) -> ParseResult<Vec<AttributeNode>> {
    let mut attributes = Vec::new();
    loop {
        take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;
        // Stop at > or /
        let check: ParseResult<&str> = peek(take_while(1.., ['>', '/'])).parse_next(parser_input);
        if check.is_ok() {
            break;
        }
        attributes.push(attribute_parser(parser_input)?);
    }
    Ok(attributes)
}

/// Parse CSS content string into stylesheet children.
/// This creates a sub-parser for the CSS content.
fn parse_stylesheet_content(source: &str, offset: u32) -> ParseResult<Vec<StyleSheetChild>> {
    let mut pos = 0;
    let mut children = Vec::new();

    loop {
        pos = skip_css_whitespace_and_comments(source, pos);
        if pos >= source.len() {
            break;
        }

        let child = css_child_parser(source, &mut pos, offset)?;
        children.push(child);
    }

    Ok(children)
}

/// Skip CSS whitespace and comments, returning new position.
pub fn skip_css_whitespace_and_comments(source: &str, mut pos: usize) -> usize {
    let bytes = source.as_bytes();
    loop {
        while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos + 1 < bytes.len() && bytes[pos] == b'/' && bytes[pos + 1] == b'*' {
            pos += 2;
            while pos + 1 < bytes.len() {
                if bytes[pos] == b'*' && bytes[pos + 1] == b'/' {
                    pos += 2;
                    break;
                }
                pos += 1;
            }
            continue;
        }
        break;
    }
    pos
}

/// Read a CSS identifier (shared between rules and selectors sub-parsers).
/// Handles escape sequences per CSS spec:
/// - Unicode escape: `\HHHHHH` (1-6 hex digits, optional trailing whitespace) → resolved codepoint
/// - Simple escape: `\X` → keeps `\X` verbatim (like reference Svelte)
pub fn read_css_ident(source: &str, pos: &mut usize) -> String {
    let bytes = source.as_bytes();
    let mut ident = String::new();

    while *pos < bytes.len() {
        let ch = bytes[*pos];
        if ch == b'\\' && *pos + 1 < bytes.len() {
            *pos += 1; // skip backslash
            // Check for unicode escape (1-6 hex digits)
            let hex_start = *pos;
            let mut hex_count = 0;
            while *pos < bytes.len() && hex_count < 6 && bytes[*pos].is_ascii_hexdigit() {
                *pos += 1;
                hex_count += 1;
            }
            if hex_count > 0 {
                // Unicode escape: resolve to codepoint
                let hex_str = &source[hex_start..*pos];
                if let Ok(cp) = u32::from_str_radix(hex_str, 16) {
                    if let Some(c) = char::from_u32(cp) {
                        ident.push(c);
                    }
                }
                // Optional trailing whitespace consumed
                if *pos < bytes.len() && bytes[*pos].is_ascii_whitespace() {
                    *pos += 1;
                }
            } else {
                // Simple escape: backslash + next char kept verbatim
                ident.push('\\');
                // Get the char (could be multi-byte)
                let ch_str = &source[*pos..];
                if let Some(c) = ch_str.chars().next() {
                    ident.push(c);
                    *pos += c.len_utf8();
                }
            }
        } else if ch.is_ascii_alphanumeric() || ch == b'-' || ch == b'_' || ch > 127 {
            // Regular ident char (ASCII or non-ASCII)
            let ch_str = &source[*pos..];
            if let Some(c) = ch_str.chars().next() {
                ident.push(c);
                *pos += c.len_utf8();
            } else {
                break;
            }
        } else {
            break;
        }
    }
    ident
}
