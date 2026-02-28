use lux_ast::template::attribute::AttributeNode;
use lux_ast::common::Span;
use lux_ast::template::element::RegularElement;
use lux_ast::template::root::FragmentNode;
use winnow::Result;

use crate::context::with_shadowroot_template;
use crate::input::Input;
use crate::parser::state::element::attribute::parse_attributes;
use crate::parser::state::element::element_body::parse_element_body;

pub fn parse_regular_element<'a>(
    input: &mut Input<'a>,
    start: usize,
    name: &'a str,
) -> Result<FragmentNode<'a>> {
    let attributes = parse_attributes(input)?;
    let shadowroot_template =
        name == "template" && has_shadowrootmode_attribute(&attributes);
    let (fragment, end) =
        with_shadowroot_template(input, shadowroot_template, |input| parse_element_body(input, name))?;

    Ok(FragmentNode::RegularElement(RegularElement {
        span: Span::new(start as u32, end as u32),
        name,
        attributes,
        fragment,
    }))
}

fn has_shadowrootmode_attribute(attributes: &[AttributeNode<'_>]) -> bool {
    attributes.iter().any(|attribute| {
        matches!(
            attribute,
            AttributeNode::Attribute(attribute) if attribute.name == "shadowrootmode"
        )
    })
}
