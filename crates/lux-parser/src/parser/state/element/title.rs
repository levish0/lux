use lux_ast::common::Span;
use lux_ast::template::element::TitleElement;
use lux_ast::template::root::FragmentNode;
use winnow::Result;
use winnow::error::ContextError;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::{literal, take_while};

use crate::input::Input;
use crate::parser::state::element::attribute::{is_tag_name_char, parse_attributes};
use crate::parser::state::fragment::parse_fragment_until;
use crate::parser::utils::helpers::skip_whitespace;

pub fn parse_title<'a>(
    input: &mut Input<'a>,
    start: usize,
    name: &'a str,
) -> Result<FragmentNode<'a>> {
    let attributes = parse_attributes(input)?;
    skip_whitespace(input);
    literal(">").parse_next(input)?;

    let fragment = parse_fragment_until(input, name)?;

    literal("</").parse_next(input)?;
    skip_whitespace(input);
    let close_name: &str = take_while(1.., is_tag_name_char).parse_next(input)?;
    if close_name != name {
        return Err(ContextError::new());
    }
    skip_whitespace(input);
    literal(">").parse_next(input)?;

    let end = input.previous_token_end();

    Ok(FragmentNode::TitleElement(TitleElement {
        span: Span::new(start as u32, end as u32),
        name,
        attributes,
        fragment,
    }))
}
