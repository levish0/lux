//! AwaitBlock visitor for analysis.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/AwaitBlock.js`
//!
//! ```js
//! export function AwaitBlock(node, context) {
//!     validate_block_not_empty(node.pending, context);
//!     validate_block_not_empty(node.then, context);
//!     validate_block_not_empty(node.catch, context);
//!     if (context.state.analysis.runes) {
//!         validate_opening_tag(node, context.state, '#');
//!         // validates :then and :catch whitespace
//!     }
//!     mark_subtree_dynamic(context.path);
//!     context.visit(node.expression, { ...context.state, expression: node.metadata.expression });
//!     if (node.pending) context.visit(node.pending);
//!     if (node.then) context.visit(node.then);
//!     if (node.catch) context.visit(node.catch);
//! }
//! ```

use lux_ast::blocks::AwaitBlock;

use crate::analyze::errors;
use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;
use crate::analyze::visitors::shared::{validate_block_not_empty, validate_opening_tag};

/// AwaitBlock visitor.
///
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/AwaitBlock.js`
pub fn visit_await_block(node: &AwaitBlock<'_>, state: &mut AnalysisState<'_, '_>, _path: &[NodeKind<'_>]) {
    // validate_block_not_empty(node.pending, context);
    validate_block_not_empty(node.pending.as_ref(), state);

    // validate_block_not_empty(node.then, context);
    validate_block_not_empty(node.then.as_ref(), state);

    // validate_block_not_empty(node.catch, context);
    validate_block_not_empty(node.catch.as_ref(), state);

    // if (context.state.analysis.runes) {
    //     validate_opening_tag(node, context.state, '#');
    //     // validates :then and :catch whitespace
    // }
    if state.analysis.runes {
        validate_opening_tag(node.span.into(), "#", state);

        // Validate :then block opening (no whitespace between { and :then)
        if let Some(ref value) = node.value {
            let start = value.span.start as usize;
            let source = state.analysis.source;
            if start >= 10 {
                let prefix = &source[start - 10..start];
                // Check for { followed by whitespace followed by :then
                // Pattern: {(\s*):then\s+$
                if let Some(brace_pos) = prefix.rfind('{') {
                    let between = &prefix[brace_pos + 1..];
                    if between.starts_with(|c: char| c.is_whitespace()) && between.contains(":then") {
                        state.analysis.error(errors::block_unexpected_character(
                            oxc_span::Span::new((start - 10) as u32, start as u32),
                            ":",
                        ));
                    }
                }
            }
        }

        // Validate :catch block opening (no whitespace between { and :catch)
        if let Some(ref error) = node.error {
            let start = error.span.start as usize;
            let source = state.analysis.source;
            if start >= 10 {
                let prefix = &source[start - 10..start];
                // Check for { followed by whitespace followed by :catch
                // Pattern: {(\s*):catch\s+$
                if let Some(brace_pos) = prefix.rfind('{') {
                    let between = &prefix[brace_pos + 1..];
                    if between.starts_with(|c: char| c.is_whitespace()) && between.contains(":catch") {
                        state.analysis.error(errors::block_unexpected_character(
                            oxc_span::Span::new((start - 10) as u32, start as u32),
                            ":",
                        ));
                    }
                }
            }
        }
    }

    // mark_subtree_dynamic(context.path);
    // TODO: implement mark_subtree_dynamic

    // context.visit calls are handled by the main visitor traversal
}
