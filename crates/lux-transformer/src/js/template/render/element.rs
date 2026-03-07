use lux_ast::template::attribute::{AttributeNode, AttributeValue};
use lux_ast::template::root::Fragment;
use lux_ast::template::tag::TextOrExpressionTag;
use lux_utils::elements::{is_load_error_element, is_void};

use crate::js::template::attribute::{is_event_attribute_name, render_static_attribute};

use super::super::StaticRenderContext;
use super::super::css_scope::scope_class_for_element;
use super::{render_fragment, render_fragment_nodes};

pub(super) fn render_regular_element(
    name: &str,
    attributes: &[AttributeNode<'_>],
    children: &Fragment<'_>,
    out: &mut String,
    has_dynamic: &mut bool,
    context: &StaticRenderContext<'_>,
    select_value: Option<&str>,
) {
    let scope_class = scope_class_for_element(context, name, attributes);
    let textarea_value = if name == "textarea" {
        find_textarea_value_attribute(attributes)
    } else {
        None
    };
    let select_own_value = if name == "select" {
        find_static_attribute_text_value(attributes, "value", has_dynamic)
    } else {
        None
    };
    let child_select_value = select_own_value.as_deref().or(select_value);
    let option_selected = name == "option"
        && select_value.is_some_and(|value| {
            option_matches_select(attributes, children, value, has_dynamic)
        });
    let mut merged_class: Option<String> = None;
    let mut class_insert_at = None;

    out.push('<');
    out.push_str(name);

    for attribute in attributes {
        if scope_class.is_some() && is_class_attribute(attribute) {
            if class_insert_at.is_none() {
                class_insert_at = Some(out.len());
            }
            merge_static_class_attribute(attribute, &mut merged_class, has_dynamic);
            continue;
        }
        if name == "textarea" && is_textarea_value_attribute(attribute) {
            continue;
        }
        if name == "select" && is_named_attribute(attribute, "value") {
            let _ = static_attribute_value_from_node(attribute, has_dynamic);
            continue;
        }
        if name == "option" && option_selected && is_named_attribute(attribute, "selected") {
            continue;
        }
        if let Some(serialized) = render_static_attribute(attribute, has_dynamic) {
            out.push(' ');
            out.push_str(&serialized);
        }
    }

    if let Some(scope_class) = scope_class {
        let target = merged_class.get_or_insert_with(String::new);
        if !target.is_empty() {
            target.push(' ');
        }
        target.push_str(scope_class);
    }

    if let Some(class_value) = merged_class.filter(|value| !value.is_empty()) {
        let serialized = format!(" class=\"{}\"", escape_text_content(&class_value));
        if let Some(index) = class_insert_at {
            out.insert_str(index, &serialized);
        } else {
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
    if option_selected && !has_boolean_attribute(attributes, "selected") {
        out.push_str(" selected");
    }

    out.push('>');

    if !is_void(name) {
        if let Some(value) = textarea_value {
            render_static_textarea_value(value, out, has_dynamic);
        } else {
            render_child_fragment(
                name,
                children,
                out,
                has_dynamic,
                context,
                child_select_value,
            );
        }
        out.push_str("</");
        out.push_str(name);
        out.push('>');
    }
}

fn render_child_fragment(
    parent_name: &str,
    children: &Fragment<'_>,
    out: &mut String,
    has_dynamic: &mut bool,
    context: &StaticRenderContext<'_>,
    select_value: Option<&str>,
) {
    if parent_name != "select" {
        render_fragment(children, out, has_dynamic, context, select_value);
        return;
    }

    let nodes = children
        .nodes
        .iter()
        .filter(|node| !is_whitespace_text_node(node))
        .collect::<Vec<_>>();
    render_fragment_nodes(&nodes, out, has_dynamic, context, select_value);
}

fn find_textarea_value_attribute<'a>(
    attributes: &'a [AttributeNode<'a>],
) -> Option<&'a AttributeValue<'a>> {
    attributes.iter().find_map(|attribute| {
        let AttributeNode::Attribute(attribute) = attribute else {
            return None;
        };
        (attribute.name == "value").then_some(&attribute.value)
    })
}

fn is_textarea_value_attribute(attribute: &AttributeNode<'_>) -> bool {
    matches!(attribute, AttributeNode::Attribute(attribute) if attribute.name == "value")
}

fn is_whitespace_text_node(node: &lux_ast::template::root::FragmentNode<'_>) -> bool {
    matches!(node, lux_ast::template::root::FragmentNode::Text(text) if text.raw.trim().is_empty())
}

fn is_class_attribute(attribute: &AttributeNode<'_>) -> bool {
    matches!(attribute, AttributeNode::Attribute(attribute) if attribute.name == "class")
}

fn is_named_attribute(attribute: &AttributeNode<'_>, name: &str) -> bool {
    matches!(attribute, AttributeNode::Attribute(attribute) if attribute.name == name)
}

fn has_boolean_attribute(attributes: &[AttributeNode<'_>], name: &str) -> bool {
    attributes
        .iter()
        .any(|attribute| is_named_attribute(attribute, name))
}

fn merge_static_class_attribute(
    attribute: &AttributeNode<'_>,
    merged_class: &mut Option<String>,
    has_dynamic: &mut bool,
) {
    let Some(value) = static_attribute_value_from_node(attribute, has_dynamic) else {
        return;
    };

    let target = merged_class.get_or_insert_with(String::new);
    if !target.is_empty() && !value.is_empty() {
        target.push(' ');
    }
    target.push_str(&value);
}

fn static_attribute_value_from_node(
    attribute: &AttributeNode<'_>,
    has_dynamic: &mut bool,
) -> Option<String> {
    let AttributeNode::Attribute(attribute) = attribute else {
        return None;
    };

    match &attribute.value {
        AttributeValue::True => Some(String::new()),
        AttributeValue::ExpressionTag(_) => {
            *has_dynamic = true;
            None
        }
        AttributeValue::Sequence(chunks) => static_text_chunks(chunks, has_dynamic),
    }
}

fn static_text_chunks(
    chunks: &[TextOrExpressionTag<'_>],
    has_dynamic: &mut bool,
) -> Option<String> {
    let mut value = String::new();
    for chunk in chunks {
        match chunk {
            TextOrExpressionTag::Text(text) => value.push_str(text.raw),
            TextOrExpressionTag::ExpressionTag(_) => {
                *has_dynamic = true;
                return None;
            }
        }
    }
    Some(value)
}

fn find_static_attribute_text_value(
    attributes: &[AttributeNode<'_>],
    name: &str,
    has_dynamic: &mut bool,
) -> Option<String> {
    attributes.iter().find_map(|attribute| {
        if is_named_attribute(attribute, name) {
            return static_attribute_value_from_node(attribute, has_dynamic);
        }
        None
    })
}

fn option_matches_select(
    attributes: &[AttributeNode<'_>],
    children: &Fragment<'_>,
    select_value: &str,
    has_dynamic: &mut bool,
) -> bool {
    find_static_attribute_text_value(attributes, "value", has_dynamic)
        .or_else(|| static_fragment_text_value(children, has_dynamic))
        .is_some_and(|value| value == select_value)
}

fn static_fragment_text_value(fragment: &Fragment<'_>, has_dynamic: &mut bool) -> Option<String> {
    let mut value = String::new();
    for node in &fragment.nodes {
        match node {
            lux_ast::template::root::FragmentNode::Text(text) => value.push_str(text.raw),
            _ => {
                *has_dynamic = true;
                return None;
            }
        }
    }
    Some(value)
}

fn render_static_textarea_value(
    value: &AttributeValue<'_>,
    out: &mut String,
    has_dynamic: &mut bool,
) {
    match value {
        AttributeValue::True => {}
        AttributeValue::ExpressionTag(_) => {
            *has_dynamic = true;
        }
        AttributeValue::Sequence(chunks) => {
            for chunk in chunks {
                match chunk {
                    TextOrExpressionTag::Text(text) => out.push_str(&escape_text_content(text.raw)),
                    TextOrExpressionTag::ExpressionTag(_) => {
                        *has_dynamic = true;
                        return;
                    }
                }
            }
        }
    }
}

fn escape_text_content(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '"' => escaped.push_str("&quot;"),
            _ => escaped.push(ch),
        }
    }
    escaped
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
