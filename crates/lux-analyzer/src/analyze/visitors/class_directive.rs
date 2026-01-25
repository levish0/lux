//! ClassDirective visitor for analysis.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/ClassDirective.js`

use lux_ast::attributes::ClassDirective;

use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;
use crate::analyze::visitors::shared::mark_subtree_dynamic;

/// ClassDirective visitor.
///
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/ClassDirective.js`
pub fn visit_class_directive(
    _node: &ClassDirective<'_>,
    state: &mut AnalysisState<'_, '_>,
    path: &[NodeKind<'_>],
) {
    mark_subtree_dynamic(state.analysis, path);
}
