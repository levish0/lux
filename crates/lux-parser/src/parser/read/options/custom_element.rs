use lux_ast::template::attribute::{Attribute, AttributeValue};
use lux_ast::template::root::{CustomElementOptions, CustomElementShadow};
use lux_ast::template::tag::TextOrExpressionTag;
use oxc_allocator::{Allocator, CloneIn};
use oxc_ast::ast::Expression;

use crate::error::{ErrorKind, ParseError};

use super::static_value::{StaticValue, get_static_value};

const RESERVED_CUSTOM_ELEMENT_TAG_NAMES: &[&str] = &[
    "annotation-xml",
    "color-profile",
    "font-face",
    "font-face-src",
    "font-face-uri",
    "font-face-format",
    "font-face-name",
    "missing-glyph",
];

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
            oxc_ast::ast::PropertyKey::StaticIdentifier(ident) => ident.name.as_str(),
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

fn parse_custom_element_shadow<'a>(
    attr: &Attribute<'a>,
    value: &Expression<'a>,
    allocator: &'a Allocator,
) -> Result<CustomElementShadow<'a>, ParseError> {
    match value.get_inner_expression() {
        Expression::StringLiteral(lit) if lit.value.as_str() == "open" => {
            Ok(CustomElementShadow::Open)
        }
        Expression::StringLiteral(lit) if lit.value.as_str() == "none" => {
            Ok(CustomElementShadow::None)
        }
        Expression::ObjectExpression(object) => Ok(CustomElementShadow::Object(
            object.as_ref().clone_in(allocator),
        )),
        _ => Err(invalid_custom_element_shadow(attr)),
    }
}

fn validate_custom_element_props(
    attr: &Attribute<'_>,
    props: &oxc_ast::ast::ObjectExpression<'_>,
) -> Result<(), ParseError> {
    for property in &props.properties {
        let Some(property) = property.as_property() else {
            return Err(invalid_custom_element_props(attr));
        };

        if property.computed {
            return Err(invalid_custom_element_props(attr));
        }

        if !matches!(property.key, oxc_ast::ast::PropertyKey::StaticIdentifier(_)) {
            return Err(invalid_custom_element_props(attr));
        }

        let Expression::ObjectExpression(definition) = property.value.get_inner_expression() else {
            return Err(invalid_custom_element_props(attr));
        };

        for definition_property in &definition.properties {
            let Some(definition_property) = definition_property.as_property() else {
                return Err(invalid_custom_element_props(attr));
            };

            if definition_property.computed {
                return Err(invalid_custom_element_props(attr));
            }

            let key = match &definition_property.key {
                oxc_ast::ast::PropertyKey::StaticIdentifier(ident) => ident.name.as_str(),
                _ => return Err(invalid_custom_element_props(attr)),
            };

            let value = definition_property.value.get_inner_expression();

            match key {
                "type" => {
                    let Expression::StringLiteral(lit) = value else {
                        return Err(invalid_custom_element_props(attr));
                    };
                    match lit.value.as_str() {
                        "String" | "Number" | "Boolean" | "Array" | "Object" => {}
                        _ => return Err(invalid_custom_element_props(attr)),
                    }
                }
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
    }

    Ok(())
}

fn get_custom_element_tag<'a>(attr: &Attribute<'a>) -> Result<&'a str, ParseError> {
    match get_static_value(attr) {
        Ok(StaticValue::String(value)) => Ok(value),
        _ => Err(invalid_custom_element_tag_name(attr)),
    }
}

fn validate_custom_element_tag_name(attr: &Attribute<'_>, tag: &str) -> Result<(), ParseError> {
    if !is_valid_custom_element_name(tag) {
        return Err(invalid_custom_element_tag_name(attr));
    }

    if RESERVED_CUSTOM_ELEMENT_TAG_NAMES.contains(&tag) {
        return Err(ParseError::with_code(
            ErrorKind::InvalidSvelteOptions,
            "svelte_options_reserved_tagname",
            attr.span,
            "Tag name is reserved",
        ));
    }

    Ok(())
}

fn is_valid_custom_element_name(tag: &str) -> bool {
    let mut chars = tag.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !first.is_ascii_lowercase() {
        return false;
    }

    let mut has_hyphen = false;

    for ch in chars {
        if ch == '-' {
            has_hyphen = true;
            continue;
        }

        if ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '.' {
            continue;
        }

        if ch.is_ascii() {
            return false;
        }

        if ch.is_whitespace()
            || ch.is_control()
            || matches!(ch, '<' | '>' | '"' | '\'' | '=' | '/' | '\\')
        {
            return false;
        }
    }

    has_hyphen
}

fn invalid_custom_element(attr: &Attribute<'_>) -> ParseError {
    ParseError::with_code(
        ErrorKind::InvalidSvelteOptions,
        "svelte_options_invalid_customelement",
        attr.span,
        "\"customElement\" must be a string literal tag name, null, or an object literal",
    )
}

fn invalid_custom_element_props(attr: &Attribute<'_>) -> ParseError {
    ParseError::with_code(
        ErrorKind::InvalidSvelteOptions,
        "svelte_options_invalid_customelement_props",
        attr.span,
        "\"props\" must be an object literal with static keys and literal values",
    )
}

fn invalid_custom_element_shadow(attr: &Attribute<'_>) -> ParseError {
    ParseError::with_code(
        ErrorKind::InvalidSvelteOptions,
        "svelte_options_invalid_customelement_shadow",
        attr.span,
        "\"shadow\" must be \"open\", \"none\", or an object literal",
    )
}

fn invalid_custom_element_tag_name(attr: &Attribute<'_>) -> ParseError {
    ParseError::with_code(
        ErrorKind::InvalidSvelteOptions,
        "svelte_options_invalid_tagname",
        attr.span,
        "Tag name must be lowercase and hyphenated",
    )
}
