use lux_ast::template::attribute::Attribute;
use oxc_ast::ast::{Expression, ObjectExpression, PropertyKey};

use crate::error::ParseError;

use super::errors::invalid_custom_element_props;

pub(super) fn validate_custom_element_props(
    attr: &Attribute<'_>,
    props: &ObjectExpression<'_>,
) -> Result<(), ParseError> {
    for property in &props.properties {
        let Some(property) = property.as_property() else {
            return Err(invalid_custom_element_props(attr));
        };

        if property.computed {
            return Err(invalid_custom_element_props(attr));
        }

        if !matches!(property.key, PropertyKey::StaticIdentifier(_)) {
            return Err(invalid_custom_element_props(attr));
        }

        let Expression::ObjectExpression(definition) = property.value.get_inner_expression() else {
            return Err(invalid_custom_element_props(attr));
        };

        validate_custom_element_prop_definition(attr, definition.as_ref())?;
    }

    Ok(())
}

fn validate_custom_element_prop_definition(
    attr: &Attribute<'_>,
    definition: &ObjectExpression<'_>,
) -> Result<(), ParseError> {
    for definition_property in &definition.properties {
        let Some(definition_property) = definition_property.as_property() else {
            return Err(invalid_custom_element_props(attr));
        };

        if definition_property.computed {
            return Err(invalid_custom_element_props(attr));
        }

        let key = match &definition_property.key {
            PropertyKey::StaticIdentifier(ident) => ident.name.as_str(),
            _ => return Err(invalid_custom_element_props(attr)),
        };

        let value = definition_property.value.get_inner_expression();
        match key {
            "type" => validate_custom_element_prop_type(attr, value)?,
            "reflect" => {
                if !matches!(value, Expression::BooleanLiteral(_)) {
                    return Err(invalid_custom_element_props(attr));
                }
            }
            "attribute" => {
                if !matches!(value, Expression::StringLiteral(_)) {
                    return Err(invalid_custom_element_props(attr));
                }
            }
            _ => return Err(invalid_custom_element_props(attr)),
        }
    }

    Ok(())
}

fn validate_custom_element_prop_type(
    attr: &Attribute<'_>,
    value: &Expression<'_>,
) -> Result<(), ParseError> {
    let Expression::StringLiteral(lit) = value else {
        return Err(invalid_custom_element_props(attr));
    };

    match lit.value.as_str() {
        "String" | "Number" | "Boolean" | "Array" | "Object" => Ok(()),
        _ => Err(invalid_custom_element_props(attr)),
    }
}
