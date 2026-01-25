//! SnippetBlock visitor for analysis.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/SnippetBlock.js`
//!
//! ```js
//! export function SnippetBlock(node, context) {
//!     context.state.analysis.snippets.add(node);
//!     validate_block_not_empty(node.body, context);
//!     if (context.state.analysis.runes) {
//!         validate_opening_tag(node, context.state, '#');
//!     }
//!     for (const arg of node.parameters) {
//!         if (arg.type === 'RestElement') {
//!             e.snippet_invalid_rest_parameter(arg);
//!         }
//!     }
//!     // ... (hoisting and conflict checks)
//! }
//! ```

use lux_ast::blocks::SnippetBlock;
use oxc_ast::ast::Expression;

use crate::analyze::analysis::SnippetBlockMeta;
use crate::analyze::errors;
use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;
use crate::analyze::visitors::shared::{validate_block_not_empty, validate_opening_tag};

/// SnippetBlock visitor.
///
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/SnippetBlock.js`
pub fn visit_snippet_block(
    node: &SnippetBlock<'_>,
    state: &mut AnalysisState<'_, '_>,
    path: &[NodeKind<'_>],
) {
    // context.state.analysis.snippets.add(node);
    state.analysis.snippets.insert(node.span.into());

    // Initialize metadata for this snippet block
    let _meta = state
        .analysis
        .snippet_block_meta
        .entry(node.span.into())
        .or_insert_with(SnippetBlockMeta::default);

    // validate_block_not_empty(node.body, context);
    validate_block_not_empty(Some(&node.body), state);

    // if (context.state.analysis.runes) {
    //     validate_opening_tag(node, context.state, '#');
    // }
    if state.analysis.runes {
        validate_opening_tag(node.span.into(), "#", state);
    }

    // for (const arg of node.parameters) {
    //     if (arg.type === 'RestElement') {
    //         e.snippet_invalid_rest_parameter(arg);
    //     }
    // }
    // In lux_ast, rest parameter is a separate field
    if let Some(ref rest) = node.rest {
        state
            .analysis
            .error(errors::snippet_invalid_rest_parameter(rest.span));
    }

    // const name = node.expression.name;
    // In lux_ast, expression is Expression enum, should be Identifier
    let snippet_name = match &node.expression {
        Expression::Identifier(id) => id.name.as_str(),
        _ => return, // Not an identifier, shouldn't happen in valid snippet
    };

    // const { path } = context;
    // const parent = path.at(-2);
    // Note: path[-1] is SnippetBlock itself, path[-2] is the parent
    let parent = if path.len() >= 2 {
        path.get(path.len() - 2)
    } else {
        None
    };

    // if (parent.type === 'Component' && parent.attributes.some(...))
    //     e.snippet_shadowing_prop(node, node.expression.name);
    // Note: Checking if snippet name shadows a prop requires access to parent's attributes
    // This would need the parent Component node, not just NodeKind
    // Full implementation would check component attributes for shadowing

    // if (node.expression.name !== 'children') return;
    if snippet_name != "children" {
        return;
    }

    // if (parent.type === 'Component' || 'SvelteComponent' || 'SvelteSelf') {
    //     if (parent.fragment.nodes.some(node => ...)) {
    //         e.snippet_conflict(node);
    //     }
    // }
    if let Some(parent_kind) = parent {
        if matches!(
            parent_kind,
            NodeKind::Component(_) | NodeKind::SvelteComponent | NodeKind::SvelteSelf
        ) {
            // Check if parent has non-snippet content
            // This requires access to parent's fragment nodes which we don't have in NodeKind
            // Full implementation would check for:
            // - node.type !== 'SnippetBlock'
            // - (node.type !== 'Text' || node.data.trim())
            // - node.type !== 'Comment'
        }
    }

    // can_hoist logic is handled elsewhere (scope analysis)
}
