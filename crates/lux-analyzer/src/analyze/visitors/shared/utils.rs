//! General utilities for analysis visitors.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/shared/utils.js`

use lux_ast::root::Fragment;
use lux_ast::node::FragmentNode;
use lux_ast::span::Span;

use crate::analyze::errors;
use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;
use crate::analyze::warnings;

/// Checks if a node path contains a component ancestor.
pub fn has_component_ancestor(path: &[NodeKind<'_>]) -> bool {
    path.iter().any(|node| {
        matches!(
            node,
            NodeKind::Component(_) | NodeKind::SvelteComponent | NodeKind::SvelteSelf
        )
    })
}

/// Checks if the path is inside a block that would create a separate template string.
pub fn is_in_conditional_block(path: &[NodeKind<'_>]) -> bool {
    path.iter().any(|node| {
        matches!(
            node,
            NodeKind::IfBlock | NodeKind::EachBlock | NodeKind::AwaitBlock | NodeKind::KeyBlock
        )
    })
}

/// Validates that the opening of a control flow block is `{` immediately followed by the expected character.
/// In legacy mode whitespace is allowed inbetween.
///
/// Reference: validate_opening_tag in utils.js
/// ```js
/// export function validate_opening_tag(node, state, expected) {
///     if (state.analysis.source[node.start + 1] !== expected) {
///         // avoid a sea of red and only mark the first few characters
///         e.block_unexpected_character({ start: node.start, end: node.start + 5 }, expected);
///     }
/// }
/// ```
pub fn validate_opening_tag(
    span: Span,
    expected: &str,
    state: &mut AnalysisState<'_, '_>,
) {
    let start = span.start;
    let source = state.analysis.source;

    // Check if the character at start + 1 is the expected character
    if let Some(actual) = source.get(start + 1..start + 1 + expected.len()) {
        if actual != expected {
            // avoid a sea of red and only mark the first few characters
            let end = (start + 5).min(source.len());
            state.analysis.error(errors::block_unexpected_character(
                oxc_span::Span::new(start as u32, end as u32),
                expected,
            ));
        }
    }
}

/// Validates that a block body is not empty (just whitespace).
/// Warns if the block has only whitespace content.
///
/// Reference: validate_block_not_empty in utils.js
pub fn validate_block_not_empty(fragment: Option<&Fragment<'_>>, state: &mut AnalysisState<'_, '_>) {
    let Some(fragment) = fragment else {
        return;
    };

    // If the block has zero nodes, someone's probably in the middle of typing
    // If the block has exactly one text node that's all whitespace, warn
    if fragment.nodes.len() == 1 {
        if let Some(FragmentNode::Text(text)) = fragment.nodes.first() {
            if text.raw.trim().is_empty() {
                state.analysis.warning(warnings::block_empty(text.span.into()));
            }
        }
    }
}
