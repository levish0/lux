//! SvelteHead visitor for analysis.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/SvelteHead.js`

use lux_ast::node::AttributeNode;
use lux_ast::elements::SvelteHead;

use crate::analyze::errors;
use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;
use crate::analyze::visitors::shared::mark_subtree_dynamic;

/// SvelteHead visitor.
///
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/SvelteHead.js`
pub fn visit_svelte_head(
    node: &SvelteHead<'_>,
    state: &mut AnalysisState<'_, '_>,
    path: &[NodeKind<'_>],
) {
    // svelte:head cannot have any attributes
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
            .error(errors::svelte_head_illegal_attribute(span.into()));
    }

    mark_subtree_dynamic(state.analysis, path);
}
