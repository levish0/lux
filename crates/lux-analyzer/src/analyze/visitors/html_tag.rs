//! HtmlTag visitor for analysis.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/HtmlTag.js`

use lux_ast::tags::HtmlTag;

use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;
use crate::analyze::visitors::shared::{mark_subtree_dynamic, validate_opening_tag};

/// HtmlTag visitor.
///
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/HtmlTag.js`
pub fn visit_html_tag(
    node: &HtmlTag<'_>,
    state: &mut AnalysisState<'_, '_>,
    path: &[NodeKind<'_>],
) {
    // if (context.state.analysis.runes) {
    //     validate_opening_tag(node, context.state, '@');
    // }
    if state.analysis.runes {
        validate_opening_tag(node.span.into(), "@", state);
    }

    // unfortunately this is necessary in order to fix invalid HTML
    // mark_subtree_dynamic(context.path);
    mark_subtree_dynamic(state.analysis, path);

    // context.next({ ...context.state, expression: node.metadata.expression });
    // Expression metadata is handled by the main visitor traversal
}
