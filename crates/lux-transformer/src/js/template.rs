use lux_ast::template::attribute::{Attribute, AttributeNode, AttributeValue};
use lux_ast::template::root::{Fragment, FragmentNode};
use lux_ast::template::tag::TextOrExpressionTag;
use lux_utils::elements::is_void;

pub(super) struct TemplateRenderResult {
    pub html: String,
    pub has_dynamic: bool,
}

pub(super) fn render_fragment_template(fragment: &Fragment<'_>) -> TemplateRenderResult {
    let mut html = String::new();
    let mut has_dynamic = false;
    render_fragment(fragment, &mut html, &mut has_dynamic);
    TemplateRenderResult { html, has_dynamic }
}

fn render_fragment(fragment: &Fragment<'_>, out: &mut String, has_dynamic: &mut bool) {
    for node in &fragment.nodes {
        render_node(node, out, has_dynamic);
    }
}

fn render_node(node: &FragmentNode<'_>, out: &mut String, has_dynamic: &mut bool) {
    match node {
        FragmentNode::Text(text) => out.push_str(text.raw),
        FragmentNode::Comment(comment) => {
            out.push_str("<!--");
            out.push_str(&sanitize_comment(comment.data));
            out.push_str("-->");
        }

        FragmentNode::RegularElement(element) => render_regular_element(
            element.name,
            &element.attributes,
            &element.fragment,
            out,
            has_dynamic,
        ),
        FragmentNode::TitleElement(element) => render_regular_element(
            element.name,
            &element.attributes,
            &element.fragment,
            out,
            has_dynamic,
        ),
        FragmentNode::SlotElement(element) => render_regular_element(
            element.name,
            &element.attributes,
            &element.fragment,
            out,
            has_dynamic,
        ),

        FragmentNode::ExpressionTag(_) => push_dynamic_marker("expression", out, has_dynamic),
        FragmentNode::HtmlTag(_) => push_dynamic_marker("html-tag", out, has_dynamic),
        FragmentNode::ConstTag(_) => push_dynamic_marker("const-tag", out, has_dynamic),
        FragmentNode::DebugTag(_) => push_dynamic_marker("debug-tag", out, has_dynamic),
        FragmentNode::RenderTag(_) => push_dynamic_marker("render-tag", out, has_dynamic),
        FragmentNode::AttachTag(_) => push_dynamic_marker("attach-tag", out, has_dynamic),
        FragmentNode::IfBlock(_) => push_dynamic_marker("if-block", out, has_dynamic),
        FragmentNode::EachBlock(_) => push_dynamic_marker("each-block", out, has_dynamic),
        FragmentNode::AwaitBlock(_) => push_dynamic_marker("await-block", out, has_dynamic),
        FragmentNode::KeyBlock(_) => push_dynamic_marker("key-block", out, has_dynamic),
        FragmentNode::SnippetBlock(_) => push_dynamic_marker("snippet-block", out, has_dynamic),

        FragmentNode::Component(_) => push_dynamic_marker("component", out, has_dynamic),
        FragmentNode::SvelteComponent(_) => push_dynamic_marker("svelte-component", out, has_dynamic),
        FragmentNode::SvelteElement(_) => push_dynamic_marker("svelte-element", out, has_dynamic),
        FragmentNode::SvelteSelf(_) => push_dynamic_marker("svelte-self", out, has_dynamic),
        FragmentNode::SvelteFragment(_) => push_dynamic_marker("svelte-fragment", out, has_dynamic),
        FragmentNode::SvelteHead(_) => push_dynamic_marker("svelte-head", out, has_dynamic),
        FragmentNode::SvelteBody(_) => push_dynamic_marker("svelte-body", out, has_dynamic),
        FragmentNode::SvelteWindow(_) => push_dynamic_marker("svelte-window", out, has_dynamic),
        FragmentNode::SvelteDocument(_) => push_dynamic_marker("svelte-document", out, has_dynamic),
        FragmentNode::SvelteBoundary(_) => push_dynamic_marker("svelte-boundary", out, has_dynamic),
        FragmentNode::SvelteOptionsRaw(_) => push_dynamic_marker("svelte-options", out, has_dynamic),
    }
}

fn render_regular_element(
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

    out.push('>');

    if !is_void(name) {
        render_fragment(children, out, has_dynamic);
        out.push_str("</");
        out.push_str(name);
        out.push('>');
    }
}

fn render_static_attribute(attribute: &AttributeNode<'_>, has_dynamic: &mut bool) -> Option<String> {
    let AttributeNode::Attribute(attribute) = attribute else {
        *has_dynamic = true;
        return None;
    };

    serialize_attribute(attribute, has_dynamic)
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

            Some(format!("{}=\"{}\"", attribute.name, escape_attribute_value(&value)))
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

fn sanitize_comment(data: &str) -> String {
    let mut sanitized = data.replace("--", "- -");
    if sanitized.ends_with('-') {
        sanitized.push(' ');
    }
    sanitized
}

fn push_dynamic_marker(kind: &str, out: &mut String, has_dynamic: &mut bool) {
    *has_dynamic = true;
    out.push_str("<!--lux:dynamic:");
    out.push_str(kind);
    out.push_str("-->");
}
