//! KeyBlock visitor for analysis.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/KeyBlock.js`
//!
//! ```js
//! export function KeyBlock(node, context) {
//!     validate_block_not_empty(node.fragment, context);
//!     if (context.state.analysis.runes) {
//!         validate_opening_tag(node, context.state, '#');
//!     }
//!     mark_subtree_dynamic(context.path);
//!     context.visit(node.expression, { ...context.state, expression: node.metadata.expression });
//!     context.visit(node.fragment);
//! }
//! ```

use lux_ast::blocks::KeyBlock;

use crate::analyze::analysis::KeyBlockMeta;
use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;
use crate::analyze::visitors::shared::{validate_block_not_empty, validate_opening_tag};

/// KeyBlock visitor.
///
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/KeyBlock.js`
pub fn visit_key_block(
    node: &KeyBlock<'_>,
    state: &mut AnalysisState<'_, '_>,
    _path: &[NodeKind<'_>],
) {
    // validate_block_not_empty(node.fragment, context);
    validate_block_not_empty(Some(&node.fragment), state);

    // if (context.state.analysis.runes) {
    //     validate_opening_tag(node, context.state, '#');
    // }
    if state.analysis.runes {
        validate_opening_tag(node.span.into(), "#", state);
    }

    // Initialize metadata for this block
    // Expression metadata will be populated when visiting the key expression
    let _meta = state
        .analysis
        .key_block_meta
        .entry(node.span.into())
        .or_insert_with(KeyBlockMeta::default);

    // mark_subtree_dynamic(context.path);
    // TODO: implement mark_subtree_dynamic

    // context.visit calls are handled by the main visitor traversal
}
