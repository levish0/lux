//! AnimateDirective visitor for analysis.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/AnimateDirective.js`

use lux_ast::attributes::AnimateDirective;

use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;

/// AnimateDirective visitor.
///
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/AnimateDirective.js`
pub fn visit_animate_directive(
    _node: &AnimateDirective<'_>,
    _state: &mut AnalysisState<'_, '_>,
    _path: &[NodeKind<'_>],
) {
    // TODO: Check for illegal await expression in metadata
    // if (node.metadata.expression.has_await) {
    //     e.illegal_await_expression(node);
    // }
}
