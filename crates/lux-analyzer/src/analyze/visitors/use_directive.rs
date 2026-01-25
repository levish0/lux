//! UseDirective visitor for analysis.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/UseDirective.js`

use lux_ast::attributes::UseDirective;

use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;
use crate::analyze::visitors::shared::mark_subtree_dynamic;

/// UseDirective visitor.
///
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/UseDirective.js`
pub fn visit_use_directive(
    _node: &UseDirective<'_>,
    state: &mut AnalysisState<'_, '_>,
    path: &[NodeKind<'_>],
) {
    mark_subtree_dynamic(state.analysis, path);

    // TODO: Check for illegal await expression in metadata
    // if (node.metadata.expression.has_await) {
    //     e.illegal_await_expression(node);
    // }
}
