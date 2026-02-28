mod errors;
mod props;
mod shadow;
mod tag;

use lux_ast::template::attribute::{Attribute, AttributeValue};
use lux_ast::template::root::CustomElementOptions;
use lux_ast::template::tag::TextOrExpressionTag;
use oxc_allocator::{Allocator, CloneIn};
use oxc_ast::ast::{Expression, PropertyKey};

use crate::error::ParseError;

use self::errors::{
    invalid_custom_element, invalid_custom_element_props, invalid_custom_element_tag_name,
};
use self::props::validate_custom_element_props;
use self::shadow::parse_custom_element_shadow;
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
            Some(TextOrExpressionTag::Text(_)) => {
                let tag = get_custom_element_tag(attr)?;
                validate_custom_element_tag_name(attr, tag)?;
                Ok(Some(CustomElementOptions {
                    tag: Some(tag),
                    shadow: None,
                    props: None,
                    extend: None,
                }))
            }
            Some(TextOrExpressionTag::ExpressionTag(tag)) => {
                parse_custom_element_expression(attr, &tag.expression, allocator)
            }
            None => Err(invalid_custom_element(attr)),
        },
    }
}

fn parse_custom_element_expression<'a>(
    attr: &Attribute<'a>,
    expression: &Expression<'a>,
    allocator: &'a Allocator,
) -> Result<Option<CustomElementOptions<'a>>, ParseError> {
    let expression = expression.get_inner_expression();

    if expression.is_null() {
        // Backward compatibility with Svelte's legacy `customElement={null}` behavior.
        return Ok(None);
    }

    let Expression::ObjectExpression(object) = expression else {
        return Err(invalid_custom_element(attr));
    };

    let mut options = CustomElementOptions {
        tag: None,
        shadow: None,
        props: None,
        extend: None,
    };

    for property in &object.properties {
        let Some(property) = property.as_property() else {
            return Err(invalid_custom_element(attr));
        };

        if property.computed {
            return Err(invalid_custom_element(attr));
        }

        let key = match &property.key {
            PropertyKey::StaticIdentifier(ident) => ident.name.as_str(),
            _ => return Err(invalid_custom_element(attr)),
        };

        let value = property.value.get_inner_expression();

        match key {
            "tag" if options.tag.is_none() => {
                let Expression::StringLiteral(tag) = value else {
                    return Err(invalid_custom_element_tag_name(attr));
                };
                validate_custom_element_tag_name(attr, tag.value.as_str())?;
                options.tag = Some(tag.value.as_str());
            }
            "props" if options.props.is_none() => {
                let Expression::ObjectExpression(props) = value else {
                    return Err(invalid_custom_element_props(attr));
                };
                validate_custom_element_props(attr, props.as_ref())?;
                options.props = Some(props.as_ref().clone_in(allocator));
            }
            "shadow" if options.shadow.is_none() => {
                options.shadow = Some(parse_custom_element_shadow(attr, value, allocator)?);
            }
            "extend" if options.extend.is_none() => {
                options.extend = Some(value.clone_in(allocator));
            }
            _ => {}
        }
    }

    Ok(Some(options))
}
