//! TransitionDirective visitor for analysis.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/TransitionDirective.js`

use lux_ast::attributes::TransitionDirective;

use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;
use crate::analyze::visitors::shared::mark_subtree_dynamic;

/// TransitionDirective visitor.
///
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/TransitionDirective.js`
pub fn visit_transition_directive(
    _node: &TransitionDirective<'_>,
    state: &mut AnalysisState<'_, '_>,
    path: &[NodeKind<'_>],
) {
    mark_subtree_dynamic(state.analysis, path);

    // TODO: Check for illegal await expression in metadata
    // if (node.metadata.expression.has_await) {
    //     e.illegal_await_expression(node);
    // }
}
