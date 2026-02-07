use lux_ast::common::Span;
use lux_ast::template::element::SvelteBoundary;
use lux_ast::template::root::FragmentNode;
use winnow::Result;

use crate::input::Input;
use crate::parser::state::element::attribute::parse_attributes;
use crate::parser::state::element::element_body::parse_element_body;

pub fn parse_svelte_boundary<'a>(
    input: &mut Input<'a>,
    start: usize,
    name: &'a str,
) -> Result<FragmentNode<'a>> {
    let attributes = parse_attributes(input)?;
    let (fragment, end) = parse_element_body(input, name)?;

    Ok(FragmentNode::SvelteBoundary(SvelteBoundary {
        span: Span::new(start as u32, end as u32),
        name,
        attributes,
        fragment,
    }))
}
