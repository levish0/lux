//! Svelte 5 semantic analyzer.
//!
//! This crate performs semantic analysis on Svelte ASTs, including:
//! - Scope analysis and binding resolution
//! - Rune detection and validation
//! - Reactivity tracking
//! - Component analysis
//!
//! ## Architecture
//!
//! The analyzer uses a two-pass approach:
//! 1. **Scope creation** (`create_scopes`): Builds the scope tree by traversing the AST
//!    and collecting all declarations and references.
//! 2. **Analysis** (`analyze`): Walks the AST with the scope information to perform
//!    semantic analysis, validation, and metadata collection.

pub mod analyze;
pub mod scope;
pub mod visitor;

pub use analyze::{analyze_component, AnalyzeOptions, ComponentAnalysis};
pub use scope::{
    create_scopes, Binding, BindingId, BindingKind, DeclarationKind, Scope, ScopeCreationResult,
    ScopeId, ScopeTree,
};
