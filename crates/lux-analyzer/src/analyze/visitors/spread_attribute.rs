//! SpreadAttribute visitor for analysis.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/SpreadAttribute.js`

use lux_ast::attributes::SpreadAttribute;

use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;
use crate::analyze::visitors::shared::mark_subtree_dynamic;

/// SpreadAttribute visitor.
///
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/SpreadAttribute.js`
pub fn visit_spread_attribute(
    _node: &SpreadAttribute<'_>,
    state: &mut AnalysisState<'_, '_>,
    path: &[NodeKind<'_>],
) {
    mark_subtree_dynamic(state.analysis, path);
}
