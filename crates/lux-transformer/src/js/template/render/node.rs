use lux_ast::template::root::FragmentNode;

use crate::js::template::marker::{push_dynamic_marker, sanitize_comment};

use super::element::render_regular_element;
use super::render_fragment;

pub(super) fn render_node(node: &FragmentNode<'_>, out: &mut String, has_dynamic: &mut bool) {
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
        FragmentNode::ConstTag(_) => {
            *has_dynamic = true;
        }
        FragmentNode::DebugTag(_) => {
            *has_dynamic = true;
        }
        FragmentNode::RenderTag(_) => push_dynamic_marker("render-tag", out, has_dynamic),
        FragmentNode::AttachTag(_) => push_dynamic_marker("attach-tag", out, has_dynamic),
        FragmentNode::IfBlock(_) => push_dynamic_marker("if-block", out, has_dynamic),
        FragmentNode::EachBlock(_) => push_dynamic_marker("each-block", out, has_dynamic),
        FragmentNode::AwaitBlock(_) => push_dynamic_marker("await-block", out, has_dynamic),
        FragmentNode::KeyBlock(_) => push_dynamic_marker("key-block", out, has_dynamic),
        FragmentNode::SnippetBlock(_) => push_dynamic_marker("snippet-block", out, has_dynamic),

        FragmentNode::Component(_) => push_dynamic_marker("component", out, has_dynamic),
        FragmentNode::SvelteComponent(_) => {
            push_dynamic_marker("svelte-component", out, has_dynamic)
        }
        FragmentNode::SvelteElement(_) => push_dynamic_marker("svelte-element", out, has_dynamic),
        FragmentNode::SvelteSelf(_) => push_dynamic_marker("svelte-self", out, has_dynamic),
        FragmentNode::SvelteFragment(element) => render_fragment(&element.fragment, out, has_dynamic),
        FragmentNode::SvelteHead(element) => render_fragment(&element.fragment, out, has_dynamic),
        FragmentNode::SvelteBody(element) => render_fragment(&element.fragment, out, has_dynamic),
        FragmentNode::SvelteWindow(element) => render_fragment(&element.fragment, out, has_dynamic),
        FragmentNode::SvelteDocument(element) => {
            render_fragment(&element.fragment, out, has_dynamic)
        }
        FragmentNode::SvelteBoundary(element) => {
            render_fragment(&element.fragment, out, has_dynamic)
        }
        FragmentNode::SvelteOptionsRaw(_) => {}
    }
}
