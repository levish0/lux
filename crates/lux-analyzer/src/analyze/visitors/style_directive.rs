//! StyleDirective visitor for analysis.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/StyleDirective.js`

use lux_ast::attributes::StyleDirective;

use crate::analyze::errors;
use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;
use crate::analyze::visitors::shared::mark_subtree_dynamic;

/// StyleDirective visitor.
///
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/StyleDirective.js`
pub fn visit_style_directive(
    node: &StyleDirective<'_>,
    state: &mut AnalysisState<'_, '_>,
    path: &[NodeKind<'_>],
) {
    // Only 'important' modifier is allowed
    if node.modifiers.len() > 1
        || (node.modifiers.len() == 1 && node.modifiers[0] != "important")
    {
        state
            .analysis
            .error(errors::style_directive_invalid_modifier(node.span.into()));
    }

    mark_subtree_dynamic(state.analysis, path);

    // TODO: Handle shorthand value (node.value === true) and binding resolution
    // TODO: Merge expression metadata from chunks
}
