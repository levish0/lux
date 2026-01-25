//! ExpressionTag visitor for analysis.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/ExpressionTag.js`

use lux_ast::tags::ExpressionTag;

use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;
use crate::analyze::visitors::shared::mark_subtree_dynamic;

/// ExpressionTag visitor.
///
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/ExpressionTag.js`
pub fn visit_expression_tag(
    _node: &ExpressionTag<'_>,
    state: &mut AnalysisState<'_, '_>,
    path: &[NodeKind<'_>],
) {
    // const in_template = context.path.at(-1)?.type === 'Fragment';
    let in_template = matches!(path.last(), Some(NodeKind::Fragment));

    // if (in_template && context.state.parent_element) {
    //     const message = is_tag_valid_with_parent('#text', context.state.parent_element);
    //     if (message) {
    //         e.node_invalid_placement(node, message);
    //     }
    // }
    // TODO: HTML tree validation (is_tag_valid_with_parent)
    let _ = in_template;

    // TODO ideally we wouldn't do this here, we'd just do it on encountering
    // an `Identifier` within the tag. But we currently need to handle `{42}` etc
    // mark_subtree_dynamic(context.path);
    mark_subtree_dynamic(state.analysis, path);

    // context.next({ ...context.state, expression: node.metadata.expression });
    // Expression metadata is handled by the main visitor traversal
}
