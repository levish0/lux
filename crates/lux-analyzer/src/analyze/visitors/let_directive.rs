//! LetDirective visitor for analysis.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/LetDirective.js`

use lux_ast::attributes::LetDirective;

use crate::analyze::errors;
use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;

/// LetDirective visitor.
///
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/LetDirective.js`
pub fn visit_let_directive(
    node: &LetDirective<'_>,
    state: &mut AnalysisState<'_, '_>,
    path: &[NodeKind<'_>],
) {
    // let: can only be used on certain elements
    let parent = path.last();

    let valid = matches!(
        parent,
        Some(NodeKind::Component(_))
            | Some(NodeKind::RegularElement(_))
            | Some(NodeKind::SlotElement)
            | Some(NodeKind::SvelteElement)
            | Some(NodeKind::SvelteComponent)
            | Some(NodeKind::SvelteSelf)
            | Some(NodeKind::SvelteFragment)
    );

    if !valid {
        state
            .analysis
            .error(errors::let_directive_invalid_placement(node.span.into()));
    }
}
