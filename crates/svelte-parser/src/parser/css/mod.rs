mod rules;
mod selectors;

use svelte_ast::css::{CssContent, StyleSheet};
use svelte_ast::node::StyleSheetChild;
use svelte_ast::span::Span;
use winnow::Result as ParseResult;
use winnow::combinator::peek;
use winnow::prelude::*;
use winnow::stream::Location;
use winnow::token::{any, literal, take_while};

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

    // Read raw content until </style>
    let styles = read_until_closing_style(parser_input)?;
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
        let next = peek(any).parse_next(parser_input)?;
        if next == '>' || next == '/' {
            break;
        }
        attributes.push(attribute_parser(parser_input)?);
    }
    Ok(attributes)
}

/// Read raw text content until `</style>` without consuming the closing tag.
fn read_until_closing_style(parser_input: &mut ParserInput) -> ParseResult<String> {
    let mut buf = String::new();
    loop {
        let check: ParseResult<&str> = peek(literal("</style>")).parse_next(parser_input);
        if check.is_ok() {
            break;
        }
        let c: char = any.parse_next(parser_input)?;
        buf.push(c);
    }
    Ok(buf)
}

/// Parse CSS content string into stylesheet children.
/// This creates a sub-parser for the CSS content.
fn parse_stylesheet_content(source: &str, offset: u32) -> ParseResult<Vec<StyleSheetChild>> {
    let mut pos = 0;
    let mut children = Vec::new();

    loop {
        // Skip whitespace and comments
        pos = skip_css_whitespace_and_comments(source, pos);
        if pos >= source.len() {
            break;
        }

        // Parse a rule or at-rule
        let child = css_child_parser(source, &mut pos, offset)?;
        children.push(child);
    }

    Ok(children)
}

/// Skip CSS whitespace and comments, returning new position.
pub(crate) fn skip_css_whitespace_and_comments(source: &str, mut pos: usize) -> usize {
    let bytes = source.as_bytes();
    loop {
        // Skip whitespace
        while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        // Skip /* ... */ comments
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
