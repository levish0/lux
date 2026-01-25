//! SvelteFragment visitor for analysis.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/SvelteFragment.js`

use lux_ast::elements::SvelteFragment;
use lux_ast::node::AttributeNode;

use crate::analyze::errors;
use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;

/// SvelteFragment visitor.
///
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/SvelteFragment.js`
pub fn visit_svelte_fragment(
    node: &SvelteFragment<'_>,
    state: &mut AnalysisState<'_, '_>,
    path: &[NodeKind<'_>],
) {
    // svelte:fragment must be a direct child of Component or SvelteComponent
    let parent = if path.len() >= 2 {
        path.get(path.len() - 2)
    } else {
        None
    };

    let valid = matches!(
        parent,
        Some(NodeKind::Component(_)) | Some(NodeKind::SvelteComponent)
    );

    if !valid {
        state
            .analysis
            .error(errors::svelte_fragment_invalid_placement(node.span.into()));
    }

    // Validate attributes - only slot and let: are allowed
    for attr in &node.attributes {
        match attr {
            AttributeNode::Attribute(a) => {
                if a.name != "slot" {
                    state
                        .analysis
                        .error(errors::svelte_fragment_invalid_attribute(a.span.into()));
                }
                // TODO: validate_slot_attribute for slot attribute
            }
            AttributeNode::LetDirective(_) => {
                // let: directives are allowed
            }
            _ => {
                let span = match attr {
                    AttributeNode::SpreadAttribute(s) => s.span,
                    AttributeNode::OnDirective(o) => o.span,
                    AttributeNode::BindDirective(b) => b.span,
                    AttributeNode::ClassDirective(c) => c.span,
                    AttributeNode::StyleDirective(s) => s.span,
                    AttributeNode::UseDirective(u) => u.span,
                    AttributeNode::TransitionDirective(t) => t.span,
                    AttributeNode::AnimateDirective(a) => a.span,
                    _ => continue,
                };
                state
                    .analysis
                    .error(errors::svelte_fragment_invalid_attribute(span.into()));
            }
        }
    }
}
