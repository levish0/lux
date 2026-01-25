//! Analysis visitors for different AST node types.
//!
//! Each visitor handles a specific node type and performs:
//! - Validation (error checking)
//! - Metadata collection
//! - Binding updates
//!
//! Structure mirrors reference:
//! `packages/svelte/src/compiler/phases/2-analyze/visitors/`

mod await_block;
mod bind_directive;
mod call_expression;
mod each_block;
mod identifier;
mod if_block;
mod key_block;
mod on_directive;
mod render_tag;
pub mod shared;
mod snippet_block;

pub use await_block::visit_await_block;
pub use bind_directive::visit_bind_directive;
pub use call_expression::visit_call_expression;
pub use each_block::visit_each_block;
pub use identifier::visit_identifier;
pub use if_block::visit_if_block;
pub use key_block::visit_key_block;
pub use on_directive::visit_on_directive;
pub use render_tag::visit_render_tag;
pub use snippet_block::visit_snippet_block;
