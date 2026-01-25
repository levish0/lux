//! Fragment-related utilities.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/shared/fragment.js`

// Note: In the reference, `mark_subtree_dynamic` mutates AST metadata.
// In our Rust implementation, we track this differently since we don't mutate the AST.
// This file is kept for structural parity with the reference.
