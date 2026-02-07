use lux_ast::common::Span;
use lux_ast::css::StyleSheet;
use lux_ast::template::attribute::Attribute;
use winnow::Result;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::{literal, take_until};

use crate::input::Input;
use crate::parser::utils::helpers::skip_whitespace;

pub fn read_style<'a>(
    input: &mut Input<'a>,
    start: usize,
    attributes: Vec<Attribute<'a>>,
) -> Result<StyleSheet<'a>> {
    let content_start = input.current_token_start();

    let content: &str = take_until(0.., "</style").parse_next(input)?;

    let content_end = input.current_token_start();

    literal("</style").parse_next(input)?;
    skip_whitespace(input);
    literal(">").parse_next(input)?;

    let end = input.previous_token_end();

    // CSS parsing is deferred — store raw content for now.
    Ok(StyleSheet {
        span: Span::new(start as u32, end as u32),
        attributes,
        children: Vec::new(),
        content_start: content_start as u32,
        content_end: content_end as u32,
        content_styles: content,
        content_comment: None,
    })
}
