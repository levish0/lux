use lux_ast::template::attribute::{Attribute, AttributeNode, AttributeValue};
use lux_ast::template::tag::TextOrExpressionTag;

pub(super) fn render_static_attribute(
    attribute: &AttributeNode<'_>,
    has_dynamic: &mut bool,
) -> Option<String> {
    match attribute {
        AttributeNode::Attribute(attribute) => {
            if is_event_attribute_name(attribute.name) {
                return None;
            }
            serialize_attribute(attribute, has_dynamic)
        }
        // bind:this requires runtime wiring in client mode.
        AttributeNode::BindDirective(directive) if directive.name == "this" => {
            *has_dynamic = true;
            None
        }
        _ => {
            *has_dynamic = true;
            None
        }
    }
}

fn serialize_attribute(attribute: &Attribute<'_>, has_dynamic: &mut bool) -> Option<String> {
    match &attribute.value {
        AttributeValue::True => Some(attribute.name.to_string()),
        AttributeValue::ExpressionTag(_) => {
            *has_dynamic = true;
            None
        }
        AttributeValue::Sequence(chunks) => {
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

            Some(format!(
                "{}=\"{}\"",
                attribute.name,
                escape_attribute_value(&value)
            ))
        }
    }
}

fn escape_attribute_value(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '"' => escaped.push_str("&quot;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

pub(super) fn is_event_attribute_name(name: &str) -> bool {
    name.len() > 2 && name.starts_with("on")
}
