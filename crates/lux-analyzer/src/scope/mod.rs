//! Scope and binding management for Svelte semantic analysis.
//!
//! This module provides the scope tree structure used during analysis.
//! It tracks variable declarations, references, and their relationships.

mod binding;
pub mod create;
mod tree;

pub use binding::{
    Assignment, Binding, BindingInitial, BindingKind, DeclarationKind, Reference,
};
pub use create::{create_scopes, ScopeCreationResult};
pub use tree::{BindingId, Scope, ScopeId, ScopeTree};
