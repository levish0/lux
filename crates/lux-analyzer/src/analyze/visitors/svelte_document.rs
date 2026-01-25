//! SvelteDocument visitor for analysis.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/SvelteDocument.js`

use lux_ast::node::AttributeNode;
use lux_ast::elements::SvelteDocument;

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

/// SvelteDocument visitor.
///
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/SvelteDocument.js`
pub fn visit_svelte_document(
    node: &SvelteDocument<'_>,
    state: &mut AnalysisState<'_, '_>,
    _path: &[NodeKind<'_>],
) {
    // svelte:document cannot have children
    disallow_children(&node.fragment, "svelte:document", state);

    // svelte:document can only have event handlers and bindings
    for attr in &node.attributes {
        match attr {
            AttributeNode::SpreadAttribute(spread) => {
                state.analysis.error(errors::svelte_document_illegal_attribute(
                    spread.span.into(),
                ));
            }
            AttributeNode::Attribute(a) if !is_event_attribute(attr) => {
                state
                    .analysis
                    .error(errors::svelte_document_illegal_attribute(a.span.into()));
            }
            AttributeNode::BindDirective(_) | AttributeNode::OnDirective(_) => {
                // These are allowed
            }
            _ => {}
        }
    }
}
