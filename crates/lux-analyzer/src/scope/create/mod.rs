//! First pass: Create the scope tree from the AST.
//!
//! This module traverses the AST and builds the scope tree by:
//! - Creating scopes for functions, blocks, and template constructs
//! - Declaring bindings for variables, functions, imports, etc.
//! - Recording references to identifiers
//! - Tracking assignments and updates

mod js;
mod svelte;

use super::{Assignment, ScopeId, ScopeTree};
use lux_ast::root::Root;
use oxc_span::Span;

/// An update (assignment or mutation) to track.
pub(crate) struct Update {
    /// The scope where the update occurred
    pub scope_id: ScopeId,
    /// The name of the binding being updated
    pub name: String,
    /// The span of the left side of the assignment
    pub left_span: Span,
    /// The span of the value being assigned (right side or the expression itself for updates)
    pub value_span: Span,
    /// Whether this is a direct assignment (x = ...) or mutation (x.prop = ...)
    pub is_direct: bool,
}

/// Result of the first pass scope creation.
pub struct ScopeCreationResult {
    /// The scope tree containing all scopes and bindings
    pub scopes: ScopeTree,
    /// Whether the AST contains a top-level await (outside of functions)
    pub has_await: bool,
}

/// Creates the scope tree from the AST.
pub fn create_scopes(root: &Root<'_>) -> ScopeCreationResult {
    use crate::visitor::SvelteVisitor;

    let mut creator = svelte::ScopeCreator::new();
    creator.visit_root(root);

    // Process updates after all declarations and references are collected
    creator.process_updates();

    ScopeCreationResult {
        scopes: creator.scopes,
        has_await: creator.has_await,
    }
}
