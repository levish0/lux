//! TitleElement visitor for analysis.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/TitleElement.js`

use lux_ast::elements::TitleElement;
use lux_ast::node::{AttributeNode, FragmentNode};

use crate::analyze::errors;
use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;

/// TitleElement visitor.
///
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/TitleElement.js`
pub fn visit_title_element(
    node: &TitleElement<'_>,
    state: &mut AnalysisState<'_, '_>,
    _path: &[NodeKind<'_>],
) {
    // title cannot have any attributes
    for attr in &node.attributes {
        let span = match attr {
            AttributeNode::Attribute(a) => a.span,
            AttributeNode::SpreadAttribute(s) => s.span,
            AttributeNode::OnDirective(o) => o.span,
            AttributeNode::BindDirective(b) => b.span,
            AttributeNode::ClassDirective(c) => c.span,
            AttributeNode::StyleDirective(s) => s.span,
            AttributeNode::UseDirective(u) => u.span,
            AttributeNode::TransitionDirective(t) => t.span,
            AttributeNode::AnimateDirective(a) => a.span,
            AttributeNode::LetDirective(l) => l.span,
            AttributeNode::AttachTag(a) => a.span,
        };
        state
            .analysis
            .error(errors::title_illegal_attribute(span.into()));
    }

    // title can only contain text and expression tags
    for child in &node.fragment.nodes {
        match child {
            FragmentNode::Text(_) | FragmentNode::ExpressionTag(_) => {
                // These are allowed
            }
            _ => {
                let span = match child {
                    FragmentNode::Comment(n) => n.span,
                    FragmentNode::HtmlTag(n) => n.span,
                    FragmentNode::ConstTag(n) => n.span,
                    FragmentNode::DebugTag(n) => n.span,
                    FragmentNode::RenderTag(n) => n.span,
                    FragmentNode::AttachTag(n) => n.span,
                    FragmentNode::IfBlock(n) => n.span,
                    FragmentNode::EachBlock(n) => n.span,
                    FragmentNode::AwaitBlock(n) => n.span,
                    FragmentNode::KeyBlock(n) => n.span,
                    FragmentNode::SnippetBlock(n) => n.span,
                    FragmentNode::RegularElement(n) => n.span,
                    FragmentNode::Component(n) => n.span,
                    FragmentNode::SvelteElement(n) => n.span,
                    FragmentNode::SvelteComponent(n) => n.span,
                    FragmentNode::SvelteSelf(n) => n.span,
                    FragmentNode::SlotElement(n) => n.span,
                    FragmentNode::SvelteHead(n) => n.span,
                    FragmentNode::SvelteBody(n) => n.span,
                    FragmentNode::SvelteWindow(n) => n.span,
                    FragmentNode::SvelteDocument(n) => n.span,
                    FragmentNode::SvelteFragment(n) => n.span,
                    FragmentNode::SvelteBoundary(n) => n.span,
                    FragmentNode::TitleElement(n) => n.span,
                    FragmentNode::SvelteOptionsRaw(n) => n.span,
                    _ => continue,
                };
                state
                    .analysis
                    .error(errors::title_invalid_content(span.into()));
            }
        }
    }
}
