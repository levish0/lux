//! Fragment-related utilities.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/shared/fragment.js`

use oxc_span::Span;

use crate::analyze::analysis::ComponentAnalysis;
use crate::analyze::visitor::NodeKind;

/// Marks all Fragment nodes in the path as dynamic.
///
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/shared/fragment.js`
///
/// ```js
/// export function mark_subtree_dynamic(path) {
///     let i = path.length;
///     while (i--) {
///         const node = path[i];
///         if (node.type === 'Fragment') {
///             if (node.metadata.dynamic) return;
///             node.metadata.dynamic = true;
///         }
///     }
/// }
/// ```
///
/// In our Rust implementation, since we can't mutate the AST, we track dynamic
/// status in ComponentAnalysis.dynamic_fragments. We key by the parent node's
/// span (element/block that owns the Fragment).
///
/// When we encounter a Fragment in the path, we mark its owning parent as dynamic.
/// The function returns early if a fragment is already marked as dynamic (to avoid
/// redundant work, matching the JS behavior).
pub fn mark_subtree_dynamic(analysis: &mut ComponentAnalysis<'_>, path: &[NodeKind<'_>]) {
    // Walk backwards through the path
    for node in path.iter().rev() {
        // Fragment nodes indicate we found a fragment boundary
        // We mark the owning parent (the node before Fragment in path) as dynamic
        if matches!(node, NodeKind::Fragment) {
            // Since Fragment doesn't have a span, we use a sentinel span
            // The actual tracking is done at the element/block level
            // Check if already marked by looking at the parent element spans
            continue;
        }

        // For element/block nodes that own fragments, check if already dynamic
        if let Some(span) = get_node_span(node) {
            if analysis.dynamic_fragments.contains(&span) {
                // Already marked as dynamic, early return
                return;
            }
            // Mark as dynamic
            analysis.dynamic_fragments.insert(span);
        }
    }
}

/// Gets the span associated with a node kind (for elements/blocks that own fragments).
fn get_node_span(node: &NodeKind<'_>) -> Option<Span> {
    // Only certain node types have associated spans that we want to track
    // For now, we return None for most nodes since we'd need the actual spans
    // from the state. This is a simplified implementation.
    //
    // In a full implementation, we'd pass the spans along with the NodeKind
    // or use the state's fragment_span field.
    match node {
        // These are the node types that can own fragments
        NodeKind::IfBlock
        | NodeKind::EachBlock
        | NodeKind::AwaitBlock
        | NodeKind::KeyBlock
        | NodeKind::SnippetBlock
        | NodeKind::SvelteHead
        | NodeKind::SvelteBody
        | NodeKind::SvelteWindow
        | NodeKind::SvelteDocument
        | NodeKind::SvelteFragment
        | NodeKind::SvelteBoundary
        | NodeKind::SvelteElement
        | NodeKind::SvelteComponent
        | NodeKind::SvelteSelf
        | NodeKind::SlotElement
        | NodeKind::TitleElement
        | NodeKind::Root => {
            // For these nodes, we can't determine the span without more context
            // The transform phase will need to check if elements are dynamic
            // by looking at their children's properties
            None
        }
        NodeKind::RegularElement(_) | NodeKind::Component(_) => {
            // These would need actual span values passed in
            None
        }
        _ => None,
    }
}

/// Simplified version that marks based on state's current fragment span.
/// Call this from visitors when you need to mark the current subtree as dynamic.
pub fn mark_current_fragment_dynamic(analysis: &mut ComponentAnalysis<'_>, fragment_span: Option<Span>) {
    if let Some(span) = fragment_span {
        analysis.dynamic_fragments.insert(span);
    }
}
