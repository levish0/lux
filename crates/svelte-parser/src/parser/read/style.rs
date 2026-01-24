use svelte_ast::css::{CssContent, StyleSheet};
use svelte_ast::node::AttributeNode;
use svelte_ast::span::Span;
use crate::parser::{ParseError, Parser};

use super::css;

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

    // Parse CSS body until </style or EOF
    let children = css::read_body(parser, |p| {
        p.match_str("</style") || p.index >= p.template.len()
    })?;

    let content_end = parser.index;

    // Consume </style\s*>
    parser.eat_required("</style")?;
    parser.allow_whitespace();
    parser.eat(">");

    // Extract only Attribute variants (style tags only have static attributes)
    let style_attributes = attributes
        .into_iter()
        .filter_map(|a| match a {
            AttributeNode::Attribute(attr) => Some(attr),
            _ => None,
        })
        .collect();

    let styles = &parser.template[content_start..content_end];

    Ok(StyleSheet {
        span: Span::new(start, parser.index),
        attributes: style_attributes,
        children,
        content: CssContent {
            start: content_start as u32,
            end: content_end as u32,
            styles,
            comment: None,
        },
    })
}
