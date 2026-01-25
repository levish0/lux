//! OnDirective visitor for analysis.

use lux_ast::attributes::OnDirective;

use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;
use crate::analyze::warnings;

/// Validates an on directive.
pub fn visit_on_directive(
    node: &OnDirective<'_>,
    state: &mut AnalysisState<'_, '_>,
    path: &[NodeKind<'_>],
) {
    let parent = path.last();

    // In runes mode, warn that on: directives are deprecated for elements
    // Don't warn on component events; these might not be under the author's control
    if state.analysis.runes {
        if let Some(p) = parent {
            if matches!(p, NodeKind::RegularElement(_) | NodeKind::SvelteElement) {
                state
                    .analysis
                    .warning(warnings::event_directive_deprecated(node.span.into(), node.name));
            }
        }
    }

    // Record first event directive node for mixed syntax checking
    if let Some(p) = parent {
        if matches!(p, NodeKind::RegularElement(_) | NodeKind::SvelteElement) {
            if state.analysis.event_directive_span.is_none() {
                state.analysis.event_directive_span = Some(node.span.into());
            }
        }
    }

    // TODO: mark_subtree_dynamic(path)
}
