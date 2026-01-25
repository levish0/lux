//! Special element utilities.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/shared/special-element.js`

use lux_ast::node::FragmentNode;
use lux_ast::root::Fragment;

use crate::analyze::errors;
use crate::analyze::state::AnalysisState;

/// Gets the span start from a FragmentNode.
fn get_node_start(node: &FragmentNode<'_>) -> usize {
    match node {
        FragmentNode::Text(n) => n.span.start,
        FragmentNode::Comment(n) => n.span.start,
        FragmentNode::ExpressionTag(n) => n.span.start,
        FragmentNode::HtmlTag(n) => n.span.start,
        FragmentNode::ConstTag(n) => n.span.start,
        FragmentNode::DebugTag(n) => n.span.start,
        FragmentNode::RenderTag(n) => n.span.start,
        FragmentNode::AttachTag(n) => n.span.start,
        FragmentNode::IfBlock(n) => n.span.start,
        FragmentNode::EachBlock(n) => n.span.start,
        FragmentNode::AwaitBlock(n) => n.span.start,
        FragmentNode::KeyBlock(n) => n.span.start,
        FragmentNode::SnippetBlock(n) => n.span.start,
        FragmentNode::RegularElement(n) => n.span.start,
        FragmentNode::Component(n) => n.span.start,
        FragmentNode::SvelteElement(n) => n.span.start,
        FragmentNode::SvelteComponent(n) => n.span.start,
        FragmentNode::SvelteSelf(n) => n.span.start,
        FragmentNode::SlotElement(n) => n.span.start,
        FragmentNode::SvelteHead(n) => n.span.start,
        FragmentNode::SvelteBody(n) => n.span.start,
        FragmentNode::SvelteWindow(n) => n.span.start,
        FragmentNode::SvelteDocument(n) => n.span.start,
        FragmentNode::SvelteFragment(n) => n.span.start,
        FragmentNode::SvelteBoundary(n) => n.span.start,
        FragmentNode::TitleElement(n) => n.span.start,
        FragmentNode::SvelteOptionsRaw(n) => n.span.start,
    }
}

/// Gets the span end from a FragmentNode.
fn get_node_end(node: &FragmentNode<'_>) -> usize {
    match node {
        FragmentNode::Text(n) => n.span.end,
        FragmentNode::Comment(n) => n.span.end,
        FragmentNode::ExpressionTag(n) => n.span.end,
        FragmentNode::HtmlTag(n) => n.span.end,
        FragmentNode::ConstTag(n) => n.span.end,
        FragmentNode::DebugTag(n) => n.span.end,
        FragmentNode::RenderTag(n) => n.span.end,
        FragmentNode::AttachTag(n) => n.span.end,
        FragmentNode::IfBlock(n) => n.span.end,
        FragmentNode::EachBlock(n) => n.span.end,
        FragmentNode::AwaitBlock(n) => n.span.end,
        FragmentNode::KeyBlock(n) => n.span.end,
        FragmentNode::SnippetBlock(n) => n.span.end,
        FragmentNode::RegularElement(n) => n.span.end,
        FragmentNode::Component(n) => n.span.end,
        FragmentNode::SvelteElement(n) => n.span.end,
        FragmentNode::SvelteComponent(n) => n.span.end,
        FragmentNode::SvelteSelf(n) => n.span.end,
        FragmentNode::SlotElement(n) => n.span.end,
        FragmentNode::SvelteHead(n) => n.span.end,
        FragmentNode::SvelteBody(n) => n.span.end,
        FragmentNode::SvelteWindow(n) => n.span.end,
        FragmentNode::SvelteDocument(n) => n.span.end,
        FragmentNode::SvelteFragment(n) => n.span.end,
        FragmentNode::SvelteBoundary(n) => n.span.end,
        FragmentNode::TitleElement(n) => n.span.end,
        FragmentNode::SvelteOptionsRaw(n) => n.span.end,
    }
}

/// Validates that a special element has no children.
/// Used for svelte:body, svelte:document, svelte:options, svelte:window.
pub fn disallow_children(
    fragment: &Fragment<'_>,
    element_name: &str,
    state: &mut AnalysisState<'_, '_>,
) {
    if !fragment.nodes.is_empty() {
        let first = fragment.nodes.first().unwrap();
        let last = fragment.nodes.last().unwrap();

        let start = get_node_start(first);
        let end = get_node_end(last);

        state.analysis.error(errors::svelte_meta_invalid_content(
            oxc_span::Span::new(start as u32, end as u32),
            element_name,
        ));
    }
}
