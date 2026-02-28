use lux_ast::common::Span;
use lux_ast::template::attribute::{Attribute, AttributeNode};

use crate::error::{ErrorKind, ParseError};

pub(super) fn extract_static_attributes<'a>(
    raw_attributes: Vec<AttributeNode<'a>>,
    span: Span,
) -> Result<Vec<Attribute<'a>>, ParseError> {
    let mut attributes: Vec<Attribute<'a>> = Vec::new();
    for attr_node in raw_attributes {
        match attr_node {
            AttributeNode::Attribute(attr) => attributes.push(attr),
            _ => {
                return Err(ParseError::with_code(
                    ErrorKind::InvalidSvelteOptions,
                    "svelte_options_invalid_attribute",
                    span,
                    "svelte:options can only have static attributes",
                ));
            }
        }
    }

    Ok(attributes)
}
