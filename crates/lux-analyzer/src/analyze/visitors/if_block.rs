//! IfBlock visitor for analysis.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/IfBlock.js`
//!
//! ```js
//! export function IfBlock(node, context) {
//!     validate_block_not_empty(node.consequent, context);
//!     validate_block_not_empty(node.alternate, context);
//!     if (context.state.analysis.runes) {
//!         validate_opening_tag(node, context.state, node.elseif ? ':' : '#');
//!     }
//!     mark_subtree_dynamic(context.path);
//!     context.visit(node.test, { ...context.state, expression: node.metadata.expression });
//!     context.visit(node.consequent);
//!     if (node.alternate) context.visit(node.alternate);
//! }
//! ```

use lux_ast::blocks::IfBlock;

use crate::analyze::analysis::IfBlockMeta;
use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;
use crate::analyze::visitors::shared::{validate_block_not_empty, validate_opening_tag};

/// IfBlock visitor.
///
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/IfBlock.js`
pub fn visit_if_block(
    node: &IfBlock<'_>,
    state: &mut AnalysisState<'_, '_>,
    _path: &[NodeKind<'_>],
) {
    // validate_block_not_empty(node.consequent, context);
    validate_block_not_empty(Some(&node.consequent), state);

    // validate_block_not_empty(node.alternate, context);
    validate_block_not_empty(node.alternate.as_ref(), state);

    // if (context.state.analysis.runes) {
    //     validate_opening_tag(node, context.state, node.elseif ? ':' : '#');
    // }
    if state.analysis.runes {
        let expected = if node.elseif { ":" } else { "#" };
        validate_opening_tag(node.span.into(), expected, state);
    }

    // Initialize metadata for this block
    // Expression metadata will be populated when visiting the test expression
    let _meta = state
        .analysis
        .if_block_meta
        .entry(node.span.into())
        .or_insert_with(IfBlockMeta::default);

    // mark_subtree_dynamic(context.path);
    // TODO: implement mark_subtree_dynamic

    // context.visit calls are handled by the main visitor traversal
}
