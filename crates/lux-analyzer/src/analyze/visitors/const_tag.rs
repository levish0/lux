//! ConstTag visitor for analysis.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/ConstTag.js`

use lux_ast::tags::ConstTag;

use crate::analyze::errors;
use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;
use crate::analyze::visitors::shared::validate_opening_tag;

/// ConstTag visitor.
///
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/ConstTag.js`
pub fn visit_const_tag(
    node: &ConstTag<'_>,
    state: &mut AnalysisState<'_, '_>,
    path: &[NodeKind<'_>],
) {
    // if (context.state.analysis.runes) {
    //     validate_opening_tag(node, context.state, '@');
    // }
    if state.analysis.runes {
        validate_opening_tag(node.span.into(), "@", state);
    }

    // const parent = context.path.at(-1);
    // const grand_parent = context.path.at(-2);
    let parent = path.last();
    let grand_parent = if path.len() >= 2 {
        path.get(path.len() - 2)
    } else {
        None
    };

    // Validate placement - @const is only allowed in specific contexts:
    // - Inside Fragment of IfBlock, EachBlock, AwaitBlock, SnippetBlock, KeyBlock
    // - Inside Fragment of SvelteFragment, Component, SvelteComponent, SvelteBoundary
    // - Inside Fragment of RegularElement or SvelteElement with a slot attribute
    //
    // if (parent?.type !== 'Fragment' || (grand_parent?.type !== 'IfBlock' && ...)) {
    //     e.const_tag_invalid_placement(node);
    // }
    let valid_placement = if let Some(NodeKind::Fragment) = parent {
        if let Some(gp) = grand_parent {
            matches!(
                gp,
                NodeKind::IfBlock
                    | NodeKind::EachBlock
                    | NodeKind::AwaitBlock
                    | NodeKind::SnippetBlock
                    | NodeKind::KeyBlock
                    | NodeKind::SvelteFragment
                    | NodeKind::SvelteBoundary
                    | NodeKind::Component(_)
                    | NodeKind::SvelteComponent
                // Note: RegularElement/SvelteElement with slot attribute would need
                // additional checking that we don't have access to here
            ) || matches!(gp, NodeKind::RegularElement(_) | NodeKind::SvelteElement)
            // TODO: Check for slot attribute on RegularElement/SvelteElement
        } else {
            false
        }
    } else {
        false
    };

    if !valid_placement {
        state
            .analysis
            .error(errors::const_tag_invalid_placement(node.span.into()));
    }

    // The declaration visiting is handled by the main visitor traversal
}
