mod errors;
mod object;
mod props;
mod shadow;
mod tag;

use lux_ast::template::attribute::{Attribute, AttributeValue};
use lux_ast::template::root::CustomElementOptions;
use lux_ast::template::tag::TextOrExpressionTag;
use oxc_allocator::Allocator;

use crate::error::ParseError;

use self::errors::invalid_custom_element;
use self::object::parse_custom_element_expression;
use self::tag::{get_custom_element_tag, validate_custom_element_tag_name};

pub(super) fn parse_custom_element_options<'a>(
    attr: &Attribute<'a>,
    allocator: &'a Allocator,
) -> Result<Option<CustomElementOptions<'a>>, ParseError> {
    match &attr.value {
        AttributeValue::True => Err(invalid_custom_element(attr)),
        AttributeValue::ExpressionTag(tag) => {
            parse_custom_element_expression(attr, &tag.expression, allocator)
        }
        AttributeValue::Sequence(chunks) => match chunks.first() {
            Some(TextOrExpressionTag::Text(_)) => parse_custom_element_string_tag(attr),
            Some(TextOrExpressionTag::ExpressionTag(tag)) => {
                parse_custom_element_expression(attr, &tag.expression, allocator)
            }
            None => Err(invalid_custom_element(attr)),
        },
    }
}

fn parse_custom_element_string_tag<'a>(
    attr: &Attribute<'a>,
) -> Result<Option<CustomElementOptions<'a>>, ParseError> {
    let tag = get_custom_element_tag(attr)?;
    validate_custom_element_tag_name(attr, tag)?;
    Ok(Some(CustomElementOptions {
        tag: Some(tag),
        shadow: None,
        props: None,
        extend: None,
    }))
}
