//! SvelteSelf visitor for analysis.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/SvelteSelf.js`

use lux_ast::elements::SvelteSelf;

use crate::analyze::errors;
use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;
use crate::analyze::visitors::shared::visit_component_like;
use crate::analyze::warnings;

/// SvelteSelf visitor.
///
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/SvelteSelf.js`
pub fn visit_svelte_self(
    node: &SvelteSelf<'_>,
    state: &mut AnalysisState<'_, '_>,
    path: &[NodeKind<'_>],
) {
    // Validate placement - must be inside IfBlock, EachBlock, Component, or SnippetBlock
    let valid = path.iter().any(|ancestor| {
        matches!(
            ancestor,
            NodeKind::IfBlock
                | NodeKind::EachBlock
                | NodeKind::Component(_)
                | NodeKind::SnippetBlock
        )
    });

    if !valid {
        state
            .analysis
            .error(errors::svelte_self_invalid_placement(node.span.into()));
    }

    // Deprecation warning in runes mode
    if state.analysis.runes {
        // Use component name or "Self" as fallback
        let name = if state.analysis.base.name.is_empty() {
            "Self"
        } else {
            &state.analysis.base.name
        };

        // For basename, we use "Self.svelte" as default
        // TODO: Get filename from options when available
        let basename = "Self.svelte";

        state.analysis.warning(warnings::svelte_self_deprecated(
            node.span.into(),
            name,
            basename,
        ));
    }

    // Use shared component validation
    visit_component_like(&node.attributes, state, path);
}
