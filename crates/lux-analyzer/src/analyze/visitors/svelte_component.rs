//! SvelteComponent visitor for analysis.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/SvelteComponent.js`

use lux_ast::elements::SvelteComponent;

use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;
use crate::analyze::visitors::shared::visit_component_like;
use crate::analyze::warnings;

/// SvelteComponent visitor.
///
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/SvelteComponent.js`
pub fn visit_svelte_component(
    node: &SvelteComponent<'_>,
    state: &mut AnalysisState<'_, '_>,
    path: &[NodeKind<'_>],
) {
    // Deprecation warning in runes mode
    if state.analysis.runes {
        state
            .analysis
            .warning(warnings::svelte_component_deprecated(node.span.into()));
    }

    // Use shared component validation
    visit_component_like(&node.attributes, state, path);
}
