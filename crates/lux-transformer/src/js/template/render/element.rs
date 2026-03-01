use lux_ast::template::attribute::AttributeNode;
use lux_ast::template::root::Fragment;
use lux_utils::elements::{is_load_error_element, is_void};

use crate::js::template::attribute::{is_event_attribute_name, render_static_attribute};

use super::render_fragment;

pub(super) fn render_regular_element(
    name: &str,
    attributes: &[AttributeNode<'_>],
    children: &Fragment<'_>,
    out: &mut String,
    has_dynamic: &mut bool,
) {
    out.push('<');
    out.push_str(name);

    for attribute in attributes {
        if let Some(serialized) = render_static_attribute(attribute, has_dynamic) {
            out.push(' ');
            out.push_str(&serialized);
        }
    }
    let (capture_onload, capture_onerror) = detect_load_error_captures(name, attributes);
    if capture_onload {
        out.push_str(" onload=\"this.__e=event\"");
    }
    if capture_onerror {
        out.push_str(" onerror=\"this.__e=event\"");
    }

    out.push('>');

    if !is_void(name) {
        render_fragment(children, out, has_dynamic);
        out.push_str("</");
        out.push_str(name);
        out.push('>');
    }
}

fn detect_load_error_captures(name: &str, attributes: &[AttributeNode<'_>]) -> (bool, bool) {
    if !is_load_error_element(name) {
        return (false, false);
    }

    let mut onload = false;
    let mut onerror = false;

    for attribute in attributes {
        match attribute {
            AttributeNode::OnDirective(directive) => match directive.name {
                "load" => onload = true,
                "error" => onerror = true,
                _ => {}
            },
            AttributeNode::Attribute(attribute) => {
                if is_event_attribute_name(attribute.name) {
                    match attribute.name {
                        "onload" => onload = true,
                        "onerror" => onerror = true,
                        _ => {}
                    }
                }
            }
            AttributeNode::SpreadAttribute(_) | AttributeNode::UseDirective(_) => {
                onload = true;
                onerror = true;
            }
            _ => {}
        }
    }

    (onload, onerror)
}
