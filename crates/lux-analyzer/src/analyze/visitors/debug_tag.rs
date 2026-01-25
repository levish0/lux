//! DebugTag visitor for analysis.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/DebugTag.js`

use lux_ast::tags::DebugTag;

use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;
use crate::analyze::visitors::shared::validate_opening_tag;

/// DebugTag visitor.
///
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/DebugTag.js`
pub fn visit_debug_tag(
    node: &DebugTag<'_>,
    state: &mut AnalysisState<'_, '_>,
    _path: &[NodeKind<'_>],
) {
    // if (context.state.analysis.runes) {
    //     validate_opening_tag(node, context.state, '@');
    // }
    if state.analysis.runes {
        validate_opening_tag(node.span.into(), "@", state);
    }

    // context.next();
    // The main visitor traversal handles visiting child nodes
}
