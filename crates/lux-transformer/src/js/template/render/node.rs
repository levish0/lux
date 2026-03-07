use lux_ast::template::root::FragmentNode;

use crate::js::template::marker::sanitize_comment;

use super::super::StaticRenderContext;
use super::element::render_regular_element;
use super::render_fragment;

pub(super) fn render_node(
    node: &FragmentNode<'_>,
    out: &mut String,
    has_dynamic: &mut bool,
    context: &StaticRenderContext<'_>,
    select_value: Option<&str>,
) {
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
            context,
            select_value,
        ),
        FragmentNode::TitleElement(element) => render_regular_element(
            element.name,
            &element.attributes,
            &element.fragment,
            out,
            has_dynamic,
            context,
            select_value,
        ),
        FragmentNode::SlotElement(_) => {
            *has_dynamic = true;
        }

        FragmentNode::ExpressionTag(_) => {
            *has_dynamic = true;
        }
        FragmentNode::HtmlTag(_) => {
            *has_dynamic = true;
        }
        FragmentNode::ConstTag(_) => {
            *has_dynamic = true;
        }
        FragmentNode::DebugTag(_) => {
            *has_dynamic = true;
        }
        FragmentNode::RenderTag(_) => {
            *has_dynamic = true;
        }
        FragmentNode::AttachTag(_) => {}
        FragmentNode::IfBlock(_) => {
            *has_dynamic = true;
        }
        FragmentNode::EachBlock(_) => {
            *has_dynamic = true;
        }
        FragmentNode::AwaitBlock(_) => {
            *has_dynamic = true;
        }
        FragmentNode::KeyBlock(block) => {
            render_fragment(&block.fragment, out, has_dynamic, context, select_value)
        }
        FragmentNode::SnippetBlock(_) => {
            *has_dynamic = true;
        }

        FragmentNode::Component(_) => {
            *has_dynamic = true;
        }
        FragmentNode::SvelteComponent(_) => {
            *has_dynamic = true;
        }
        FragmentNode::SvelteElement(_) => {
            *has_dynamic = true;
        }
        FragmentNode::SvelteSelf(_) => {
            *has_dynamic = true;
        }
        FragmentNode::SvelteFragment(element) => {
            render_fragment(&element.fragment, out, has_dynamic, context, select_value)
        }
        FragmentNode::SvelteHead(_) => {}
        FragmentNode::SvelteBody(element) => {
            render_fragment(&element.fragment, out, has_dynamic, context, select_value)
        }
        FragmentNode::SvelteWindow(element) => {
            render_fragment(&element.fragment, out, has_dynamic, context, select_value)
        }
        FragmentNode::SvelteDocument(element) => {
            render_fragment(&element.fragment, out, has_dynamic, context, select_value)
        }
        FragmentNode::SvelteBoundary(element) => {
            render_fragment(&element.fragment, out, has_dynamic, context, select_value)
        }
        FragmentNode::SvelteOptionsRaw(_) => {}
    }
}
