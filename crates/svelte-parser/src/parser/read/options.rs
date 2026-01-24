use std::collections::HashMap;

use oxc_allocator::{Allocator, CloneIn};
use oxc_ast::ast::{Expression, ObjectPropertyKind, PropertyKey};
use regex::Regex;
use std::sync::LazyLock;

use svelte_ast::attributes::{Attribute, AttributeSequenceValue, AttributeValue};
use svelte_ast::elements::SvelteOptionsRaw;
use svelte_ast::node::AttributeNode;
use svelte_ast::root::{
    CssMode, CustomElementOptions, CustomElementProp, Namespace, PropType, ShadowMode,
    SvelteOptions,
};

use crate::error::ErrorKind;
use crate::parser::ParseError;

// https://html.spec.whatwg.org/multipage/custom-elements.html#valid-custom-element-name
static REGEX_VALID_TAG_NAME: LazyLock<Regex> = LazyLock::new(|| {
    let tag_name_char = concat!(
        "[a-z0-9_\\.\\-",
        "\u{B7}",
        "\u{C0}-\u{D6}",
        "\u{D8}-\u{F6}",
        "\u{F8}-\u{037D}",
        "\u{037F}-\u{1FFF}",
        "\u{200C}-\u{200D}",
        "\u{203F}-\u{2040}",
        "\u{2070}-\u{218F}",
        "\u{2C00}-\u{2FEF}",
        "\u{3001}-\u{D7FF}",
        "\u{F900}-\u{FDCF}",
        "\u{FDF0}-\u{FFFD}",
        "\u{10000}-\u{EFFFF}",
        "]"
    );
    Regex::new(&format!("^[a-z]{tag_name_char}*-{tag_name_char}*$")).unwrap()
});

const RESERVED_TAG_NAMES: &[&str] = &[
    "annotation-xml",
    "color-profile",
    "font-face",
    "font-face-src",
    "font-face-uri",
    "font-face-format",
    "font-face-name",
    "missing-glyph",
];

fn validate_tag(tag: Option<&str>, error_pos: usize, errors: &mut Vec<ParseError>) {
    let Some(tag) = tag else {
        errors.push(ParseError {
            kind: ErrorKind::SvelteOptionsInvalidTagName,
            position: error_pos,
            message: "Invalid custom element tag name".to_string(),
        });
        return;
    };
    if !tag.is_empty() {
        if !REGEX_VALID_TAG_NAME.is_match(tag) {
            errors.push(ParseError {
                kind: ErrorKind::SvelteOptionsInvalidTagName,
                position: error_pos,
                message: "Invalid custom element tag name".to_string(),
            });
        } else if RESERVED_TAG_NAMES.contains(&tag) {
            errors.push(ParseError {
                kind: ErrorKind::SvelteOptionsInvalidTagName,
                position: error_pos,
                message: "Reserved custom element tag name".to_string(),
            });
        }
    }
}

/// Extract structured `SvelteOptions` from a raw `<svelte:options>` element.
/// Reference: `packages/svelte/src/compiler/phases/1-parse/read/options.js`
pub fn read_options<'a>(
    node: SvelteOptionsRaw<'a>,
    errors: &mut Vec<ParseError>,
    allocator: &'a Allocator,
) -> SvelteOptions<'a> {
    let span = node.span;
    let attributes = node.attributes;

    let mut runes = None;
    let mut immutable = None;
    let mut accessors = None;
    let mut preserve_whitespace = None;
    let mut namespace = None;
    let mut css = None;
    let mut custom_element = None;

    for attr in &attributes {
        let AttributeNode::Attribute(attribute) = attr else {
            errors.push(ParseError {
                kind: ErrorKind::SvelteOptionsInvalidAttribute,
                position: span.start,
                message: "Invalid attribute on <svelte:options>".to_string(),
            });
            continue;
        };

        match attribute.name {
            "runes" => match get_boolean_value(attribute) {
                Some(val) => runes = Some(val),
                None => errors.push(ParseError {
                    kind: ErrorKind::SvelteOptionsInvalidAttributeValue,
                    position: attribute.span.start,
                    message: "Expected true or false".to_string(),
                }),
            },
            "immutable" => match get_boolean_value(attribute) {
                Some(val) => immutable = Some(val),
                None => errors.push(ParseError {
                    kind: ErrorKind::SvelteOptionsInvalidAttributeValue,
                    position: attribute.span.start,
                    message: "Expected true or false".to_string(),
                }),
            },
            "accessors" => match get_boolean_value(attribute) {
                Some(val) => accessors = Some(val),
                None => errors.push(ParseError {
                    kind: ErrorKind::SvelteOptionsInvalidAttributeValue,
                    position: attribute.span.start,
                    message: "Expected true or false".to_string(),
                }),
            },
            "preserveWhitespace" => match get_boolean_value(attribute) {
                Some(val) => preserve_whitespace = Some(val),
                None => errors.push(ParseError {
                    kind: ErrorKind::SvelteOptionsInvalidAttributeValue,
                    position: attribute.span.start,
                    message: "Expected true or false".to_string(),
                }),
            },
            "namespace" => match get_static_value(attribute) {
                Some(value) => {
                    namespace = match value {
                        "svg" | "http://www.w3.org/2000/svg" => Some(Namespace::Svg),
                        "mathml" | "http://www.w3.org/1998/Math/MathML" => Some(Namespace::Mathml),
                        "html" => Some(Namespace::Html),
                        _ => {
                            errors.push(ParseError {
                                kind: ErrorKind::SvelteOptionsInvalidAttributeValue,
                                position: attribute.span.start,
                                message: r#"Expected "html", "mathml" or "svg""#.to_string(),
                            });
                            None
                        }
                    };
                }
                None => {
                    errors.push(ParseError {
                        kind: ErrorKind::SvelteOptionsInvalidAttributeValue,
                        position: attribute.span.start,
                        message: r#"Expected "html", "mathml" or "svg""#.to_string(),
                    });
                }
            },
            "css" => match get_static_value(attribute) {
                Some("injected") => {
                    css = Some(CssMode::Injected);
                }
                _ => {
                    errors.push(ParseError {
                        kind: ErrorKind::SvelteOptionsInvalidAttributeValue,
                        position: attribute.span.start,
                        message: r#"Expected "injected""#.to_string(),
                    });
                }
            },
            "customElement" => {
                custom_element = read_custom_element(attribute, errors, allocator);
            }
            "tag" => {
                errors.push(ParseError {
                    kind: ErrorKind::SvelteOptionsDeprecatedTag,
                    position: attribute.span.start,
                    message: "The 'tag' option is deprecated. Use 'customElement' instead."
                        .to_string(),
                });
            }
            _ => {
                errors.push(ParseError {
                    kind: ErrorKind::SvelteOptionsUnknownAttribute,
                    position: attribute.span.start,
                    message: format!("Unknown attribute '{}'", attribute.name),
                });
            }
        }
    }

    SvelteOptions {
        span,
        runes,
        immutable,
        accessors,
        preserve_whitespace,
        namespace,
        css,
        custom_element,
        attributes,
    }
}

/// Parse the `customElement` attribute value into `CustomElementOptions`.
fn read_custom_element<'a>(
    attribute: &Attribute<'a>,
    errors: &mut Vec<ParseError>,
    allocator: &'a Allocator,
) -> Option<CustomElementOptions<'a>> {
    let error_pos = attribute.span.start;

    let expr_tag = match &attribute.value {
        AttributeValue::True => {
            errors.push(ParseError {
                kind: ErrorKind::SvelteOptionsInvalidCustomElement,
                position: error_pos,
                message: "Invalid customElement value".to_string(),
            });
            return None;
        }
        AttributeValue::ExpressionTag(tag) => tag,
        AttributeValue::Sequence(items) => {
            if items.len() == 1 {
                match &items[0] {
                    AttributeSequenceValue::Text(text) => {
                        validate_tag(Some(text.data), error_pos, errors);
                        return Some(CustomElementOptions {
                            tag: Some(text.data),
                            shadow: None,
                            props: None,
                            extend: None,
                        });
                    }
                    AttributeSequenceValue::ExpressionTag(tag) => tag,
                }
            } else {
                errors.push(ParseError {
                    kind: ErrorKind::SvelteOptionsInvalidCustomElement,
                    position: error_pos,
                    message: "Invalid customElement value".to_string(),
                });
                return None;
            }
        }
    };

    match &expr_tag.expression {
        // customElement={null} - backwards compat, skip
        Expression::NullLiteral(_) => None,
        // customElement={{ tag: "...", ... }}
        Expression::ObjectExpression(obj) => {
            let mut ce = CustomElementOptions {
                tag: None,
                shadow: None,
                props: None,
                extend: None,
            };

            // Collect properties as (name, &Expression) pairs, matching reference
            let mut properties: Vec<(&str, &Expression<'a>)> = Vec::new();
            for prop_kind in &obj.properties {
                let ObjectPropertyKind::ObjectProperty(property) = prop_kind else {
                    errors.push(ParseError {
                        kind: ErrorKind::SvelteOptionsInvalidCustomElement,
                        position: error_pos,
                        message: "Invalid customElement value".to_string(),
                    });
                    continue;
                };
                if property.computed {
                    errors.push(ParseError {
                        kind: ErrorKind::SvelteOptionsInvalidCustomElement,
                        position: error_pos,
                        message: "Invalid customElement value".to_string(),
                    });
                    continue;
                }
                let Some(key_name) = prop_key_identifier(&property.key) else {
                    errors.push(ParseError {
                        kind: ErrorKind::SvelteOptionsInvalidCustomElement,
                        position: error_pos,
                        message: "Invalid customElement value".to_string(),
                    });
                    continue;
                };
                properties.push((key_name, &property.value));
            }

            // Process tag
            if let Some((_, value)) = properties.iter().find(|(n, _)| *n == "tag") {
                if let Expression::StringLiteral(lit) = value {
                    let tag_value = lit.value.as_str();
                    validate_tag(Some(tag_value), error_pos, errors);
                    ce.tag = Some(tag_value);
                } else {
                    errors.push(ParseError {
                        kind: ErrorKind::SvelteOptionsInvalidTagName,
                        position: error_pos,
                        message: "Invalid custom element tag name".to_string(),
                    });
                }
            }

            // Process shadow
            if let Some((_, value)) = properties.iter().find(|(n, _)| *n == "shadow") {
                if let Expression::StringLiteral(lit) = value {
                    ce.shadow = match lit.value.as_str() {
                        "open" => Some(ShadowMode::Open),
                        "none" => Some(ShadowMode::None),
                        _ => {
                            errors.push(ParseError {
                                kind: ErrorKind::SvelteOptionsInvalidCustomElementShadow,
                                position: error_pos,
                                message: r#"Expected "open" or "none""#.to_string(),
                            });
                            None
                        }
                    };
                }
            }

            // Process props
            if let Some((_, value)) = properties.iter().find(|(n, _)| *n == "props") {
                if let Expression::ObjectExpression(props_obj) = value {
                    let mut props_map: HashMap<&'a str, CustomElementProp<'a>> = HashMap::new();
                    for prop_kind in &props_obj.properties {
                        let ObjectPropertyKind::ObjectProperty(prop) = prop_kind else {
                            errors.push(ParseError {
                                kind: ErrorKind::SvelteOptionsInvalidCustomElementProps,
                                position: error_pos,
                                message: "Invalid customElement props".to_string(),
                            });
                            continue;
                        };
                        if prop.computed {
                            errors.push(ParseError {
                                kind: ErrorKind::SvelteOptionsInvalidCustomElementProps,
                                position: error_pos,
                                message: "Invalid customElement props".to_string(),
                            });
                            continue;
                        }
                        let Some(prop_name) = prop_key_identifier(&prop.key) else {
                            errors.push(ParseError {
                                kind: ErrorKind::SvelteOptionsInvalidCustomElementProps,
                                position: error_pos,
                                message: "Invalid customElement props".to_string(),
                            });
                            continue;
                        };
                        if let Expression::ObjectExpression(prop_obj) = &prop.value {
                            let ce_prop = read_custom_element_prop(prop_obj, errors, error_pos);
                            props_map.insert(prop_name, ce_prop);
                        } else {
                            errors.push(ParseError {
                                kind: ErrorKind::SvelteOptionsInvalidCustomElementProps,
                                position: error_pos,
                                message: "Invalid customElement props".to_string(),
                            });
                        }
                    }
                    ce.props = Some(props_map);
                } else {
                    errors.push(ParseError {
                        kind: ErrorKind::SvelteOptionsInvalidCustomElementProps,
                        position: error_pos,
                        message: "Invalid customElement props".to_string(),
                    });
                }
            }

            // Process extend - clone the expression into the arena
            if let Some((_, value)) = properties.iter().find(|(n, _)| *n == "extend") {
                ce.extend = Some(value.clone_in(allocator));
            }

            Some(ce)
        }
        // customElement={"tag-name"} via ExpressionTag with StringLiteral
        Expression::StringLiteral(lit) => {
            let tag_value = lit.value.as_str();
            validate_tag(Some(tag_value), error_pos, errors);
            Some(CustomElementOptions {
                tag: Some(tag_value),
                shadow: None,
                props: None,
                extend: None,
            })
        }
        _ => {
            errors.push(ParseError {
                kind: ErrorKind::SvelteOptionsInvalidCustomElement,
                position: error_pos,
                message: "Invalid customElement value".to_string(),
            });
            None
        }
    }
}

/// Read a single prop definition: { type: "String", reflect: true, attribute: "my-attr" }
fn read_custom_element_prop<'a>(
    obj: &oxc_ast::ast::ObjectExpression<'a>,
    errors: &mut Vec<ParseError>,
    error_pos: usize,
) -> CustomElementProp<'a> {
    let mut prop = CustomElementProp {
        attribute: None,
        reflect: None,
        prop_type: None,
    };

    for prop_kind in &obj.properties {
        let ObjectPropertyKind::ObjectProperty(property) = prop_kind else {
            errors.push(ParseError {
                kind: ErrorKind::SvelteOptionsInvalidCustomElementProps,
                position: error_pos,
                message: "Invalid customElement props".to_string(),
            });
            continue;
        };
        if property.computed {
            errors.push(ParseError {
                kind: ErrorKind::SvelteOptionsInvalidCustomElementProps,
                position: error_pos,
                message: "Invalid customElement props".to_string(),
            });
            continue;
        }
        let Some(key_name) = prop_key_identifier(&property.key) else {
            errors.push(ParseError {
                kind: ErrorKind::SvelteOptionsInvalidCustomElementProps,
                position: error_pos,
                message: "Invalid customElement props".to_string(),
            });
            continue;
        };

        match key_name {
            "type" => {
                if let Expression::StringLiteral(lit) = &property.value {
                    prop.prop_type = match lit.value.as_str() {
                        "Array" => Some(PropType::Array),
                        "Boolean" => Some(PropType::Boolean),
                        "Number" => Some(PropType::Number),
                        "Object" => Some(PropType::Object),
                        "String" => Some(PropType::String),
                        _ => {
                            errors.push(ParseError {
                                kind: ErrorKind::SvelteOptionsInvalidCustomElementProps,
                                position: error_pos,
                                message: "Invalid customElement prop type".to_string(),
                            });
                            None
                        }
                    };
                } else {
                    errors.push(ParseError {
                        kind: ErrorKind::SvelteOptionsInvalidCustomElementProps,
                        position: error_pos,
                        message: "Invalid customElement props".to_string(),
                    });
                }
            }
            "reflect" => {
                if let Expression::BooleanLiteral(lit) = &property.value {
                    prop.reflect = Some(lit.value);
                } else {
                    errors.push(ParseError {
                        kind: ErrorKind::SvelteOptionsInvalidCustomElementProps,
                        position: error_pos,
                        message: "Invalid customElement props".to_string(),
                    });
                }
            }
            "attribute" => {
                if let Expression::StringLiteral(lit) = &property.value {
                    prop.attribute = Some(lit.value.as_str());
                } else {
                    errors.push(ParseError {
                        kind: ErrorKind::SvelteOptionsInvalidCustomElementProps,
                        position: error_pos,
                        message: "Invalid customElement props".to_string(),
                    });
                }
            }
            _ => {
                errors.push(ParseError {
                    kind: ErrorKind::SvelteOptionsInvalidCustomElementProps,
                    position: error_pos,
                    message: "Invalid customElement props".to_string(),
                });
            }
        }
    }

    prop
}

/// Extract the name from a property key (must be Identifier per reference).
fn prop_key_identifier<'a>(key: &PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.as_str()),
        _ => None,
    }
}

/// Get a static string value from an attribute (text content or string literal expression).
fn get_static_value<'a>(attribute: &Attribute<'a>) -> Option<&'a str> {
    match &attribute.value {
        AttributeValue::True => None,
        AttributeValue::ExpressionTag(tag) => match &tag.expression {
            Expression::StringLiteral(lit) => Some(lit.value.as_str()),
            _ => None,
        },
        AttributeValue::Sequence(items) => {
            if items.len() != 1 {
                return None;
            }
            match &items[0] {
                AttributeSequenceValue::Text(text) => Some(text.data),
                AttributeSequenceValue::ExpressionTag(tag) => match &tag.expression {
                    Expression::StringLiteral(lit) => Some(lit.value.as_str()),
                    _ => None,
                },
            }
        }
    }
}

/// Get a boolean value from an attribute (e.g., `runes` or `runes={true}`).
fn get_boolean_value(attribute: &Attribute) -> Option<bool> {
    match &attribute.value {
        AttributeValue::True => Some(true),
        AttributeValue::ExpressionTag(tag) => match &tag.expression {
            Expression::BooleanLiteral(lit) => Some(lit.value),
            _ => None,
        },
        AttributeValue::Sequence(items) => {
            if items.len() == 1 {
                if let AttributeSequenceValue::ExpressionTag(tag) = &items[0] {
                    if let Expression::BooleanLiteral(lit) = &tag.expression {
                        return Some(lit.value);
                    }
                }
            }
            None
        }
    }
}
