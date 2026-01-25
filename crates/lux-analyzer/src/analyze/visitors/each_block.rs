//! EachBlock visitor for analysis.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/EachBlock.js`
//!
//! ```js
//! export function EachBlock(node, context) {
//!     validate_opening_tag(node, context.state, '#');
//!     validate_block_not_empty(node.body, context);
//!     validate_block_not_empty(node.fallback, context);
//!     const id = node.context;
//!     if (id?.type === 'Identifier' && (id.name === '$state' || id.name === '$derived')) {
//!         e.state_invalid_placement(node, id.name);
//!     }
//!     if (node.key) {
//!         node.metadata.keyed =
//!             node.key.type !== 'Identifier' || !node.index || node.key.name !== node.index;
//!     }
//!     if (node.metadata.keyed && !node.context) {
//!         e.each_key_without_as(node.key);
//!     }
//!     // ... (visit expressions and legacy mode handling)
//!     mark_subtree_dynamic(context.path);
//! }
//! ```

use lux_ast::blocks::EachBlock;
use oxc_ast::ast::{BindingPattern, Expression};

use crate::analyze::errors;
use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;
use crate::analyze::visitors::shared::{validate_block_not_empty, validate_opening_tag};

/// EachBlock visitor.
///
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/EachBlock.js`
pub fn visit_each_block(node: &EachBlock<'_>, state: &mut AnalysisState<'_, '_>, _path: &[NodeKind<'_>]) {
    // validate_opening_tag(node, context.state, '#');
    validate_opening_tag(node.span.into(), "#", state);

    // validate_block_not_empty(node.body, context);
    validate_block_not_empty(Some(&node.body), state);

    // validate_block_not_empty(node.fallback, context);
    validate_block_not_empty(node.fallback.as_ref(), state);

    // const id = node.context;
    // if (id?.type === 'Identifier' && (id.name === '$state' || id.name === '$derived')) {
    //     e.state_invalid_placement(node, id.name);
    // }
    if let Some(ref context) = node.context {
        if let BindingPattern::BindingIdentifier(id) = &context.pattern {
            let name = id.name.as_str();
            if name == "$state" || name == "$derived" {
                state.analysis.error(errors::state_invalid_placement(context.span, name));
            }
        }
    }

    // if (node.key) {
    //     // treat `{#each items as item, i (i)}` as a normal indexed block, everything else as keyed
    //     node.metadata.keyed =
    //         node.key.type !== 'Identifier' || !node.index || node.key.name !== node.index;
    // }
    // Note: metadata.keyed is set during parsing or should be computed here
    // For now we compute the "keyed" status for validation purposes
    let is_keyed = if let Some(ref key) = node.key {
        match key {
            Expression::Identifier(id) => {
                // keyed if: key is not identifier, OR no index, OR key.name != index
                node.index.is_none() || node.index != Some(id.name.as_str())
            }
            _ => true, // not an identifier, so it's keyed
        }
    } else {
        false
    };

    // if (node.metadata.keyed && !node.context) {
    //     e.each_key_without_as(node.key);
    // }
    if is_keyed && node.context.is_none() {
        if let Some(ref key) = node.key {
            let key_span = match key {
                Expression::Identifier(id) => id.span,
                _ => oxc_span::Span::new(node.span.start as u32, node.span.end as u32),
            };
            state.analysis.error(errors::each_key_without_as(key_span));
        }
    }

    // The rest of the reference (context.visit, legacy mode handling, mark_subtree_dynamic)
    // is handled by the main visitor traversal and other analysis passes
}
