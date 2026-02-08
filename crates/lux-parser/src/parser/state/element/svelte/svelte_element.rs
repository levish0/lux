use lux_ast::common::Span;
use lux_ast::template::element::SvelteElement;
use lux_ast::template::root::FragmentNode;
use winnow::Result;

use crate::input::Input;
use crate::parser::state::element::attribute::parse_attributes;
use crate::parser::state::element::element_body::{extract_this_expression, parse_element_body};

pub fn parse_svelte_element<'a>(
    input: &mut Input<'a>,
    start: usize,
    name: &'a str,
) -> Result<FragmentNode<'a>> {
    let mut attributes = parse_attributes(input)?;
    let tag = extract_this_expression(&mut attributes)?;
    let (fragment, end) = parse_element_body(input, name)?;

    Ok(FragmentNode::SvelteElement(SvelteElement {
        span: Span::new(start as u32, end as u32),
        name,
        tag,
        attributes,
        fragment,
    }))
}
