//! Fragment-related utilities.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/shared/fragment.js`

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
/// TODO: Implement proper fragment dynamic marking.
/// Currently Fragment.metadata.dynamic is stored on the AST node itself,
/// but marking requires mutable access to parent Fragment nodes which
/// isn't available through the NodeKind path. Options:
/// 1. Change path to hold mutable references
/// 2. Use a post-pass to mark fragments based on children
/// 3. Use a visitor callback pattern
///
/// For now, this is a placeholder that doesn't actually mark anything.
/// The transform phase will need to determine fragment dynamism differently.
pub fn mark_subtree_dynamic(_analysis: &mut ComponentAnalysis<'_>, _path: &[NodeKind<'_>]) {
    // TODO: Implement proper fragment dynamic marking
    // This would need mutable access to parent Fragment nodes,
    // which isn't currently available through NodeKind path.
}
