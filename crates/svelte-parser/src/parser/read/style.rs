use std::sync::LazyLock;

use regex::Regex;

use svelte_ast::css::{CssContent, StyleSheet};
use svelte_ast::node::AttributeNode;
use svelte_ast::span::Span;
use crate::error::ErrorKind::ElementUnclosed;
use crate::parser::{ParseError, Parser};

/// Regex to find `</style` (optionally with whitespace) followed by `>`.
static REGEX_CLOSING_STYLE_TAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"</style\s*>").unwrap());

/// Regex to match `</style...>` at the start of remaining input.
static REGEX_STARTS_WITH_CLOSING_STYLE_TAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^</style\s*>").unwrap());

/// Read the content of a `<style>` tag and parse it as CSS.
/// Port of reference `read/style.js`.
///
/// `start` is the byte offset of the opening `<` of `<style>`.
/// `parser.index` should be just after the `>` of the opening tag.
pub fn read_style<'a>(
    parser: &mut Parser<'a>,
    start: usize,
    attributes: Vec<AttributeNode<'a>>,
) -> Result<StyleSheet<'a>, ParseError> {
    let content_start = parser.index;

    // Read content until </style>
    let styles = parser.read_until(&REGEX_CLOSING_STYLE_TAG);
    let content_end = parser.index;

    if parser.index >= parser.template.len() {
        if !parser.loose {
            return Err(parser.error(
                ElementUnclosed,
                parser.template.len(),
                "'<style>' was not closed".to_string(),
            ));
        }
    }

    // Consume the </style> tag
    parser.read(&REGEX_STARTS_WITH_CLOSING_STYLE_TAG);

    // Extract only Attribute variants (style tags only have static attributes)
    let style_attributes = attributes
        .into_iter()
        .filter_map(|a| match a {
            AttributeNode::Attribute(attr) => Some(attr),
            _ => None,
        })
        .collect();

    // TODO: Parse CSS rules into children (full CSS parser)
    // For now, we store the raw styles and return empty children.
    let children = Vec::new();

    Ok(StyleSheet {
        span: Span::new(start, parser.index),
        attributes: style_attributes,
        children,
        content: CssContent {
            start: content_start as u32,
            end: content_end as u32,
            styles: styles.to_string(),
            comment: None,
        },
    })
}
