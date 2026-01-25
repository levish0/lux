//! SvelteBody visitor for analysis.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/SvelteBody.js`

use lux_ast::node::AttributeNode;
use lux_ast::elements::SvelteBody;

use crate::analyze::errors;
use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;
use crate::analyze::visitors::shared::disallow_children;

/// Checks if an attribute is an event attribute (starts with "on").
fn is_event_attribute(attr: &AttributeNode<'_>) -> bool {
    match attr {
        AttributeNode::Attribute(a) => a.name.starts_with("on"),
        AttributeNode::OnDirective(_) => true,
        _ => false,
    }
}

/// SvelteBody visitor.
///
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/SvelteBody.js`
pub fn visit_svelte_body(
    node: &SvelteBody<'_>,
    state: &mut AnalysisState<'_, '_>,
    _path: &[NodeKind<'_>],
) {
    // svelte:body cannot have children
    disallow_children(&node.fragment, "svelte:body", state);

    // svelte:body can only have event handlers
    for attr in &node.attributes {
        match attr {
            AttributeNode::SpreadAttribute(spread) => {
                state
                    .analysis
                    .error(errors::svelte_body_illegal_attribute(spread.span.into()));
            }
            AttributeNode::Attribute(a) if !is_event_attribute(attr) => {
                state
                    .analysis
                    .error(errors::svelte_body_illegal_attribute(a.span.into()));
            }
            _ => {}
        }
    }
}
