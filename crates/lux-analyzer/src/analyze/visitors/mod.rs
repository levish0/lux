//! Analysis visitors for different AST node types.
//!
//! Each visitor handles a specific node type and performs:
//! - Validation (error checking)
//! - Metadata collection
//! - Binding updates
//!
//! Structure mirrors reference:
//! `packages/svelte/src/compiler/phases/2-analyze/visitors/`

mod animate_directive;
mod attribute;
mod await_block;
mod bind_directive;
mod call_expression;
mod class_directive;
mod component;
mod const_tag;
mod debug_tag;
mod each_block;
mod expression_tag;
mod html_tag;
mod identifier;
mod if_block;
mod key_block;
mod let_directive;
mod on_directive;
mod regular_element;
mod render_tag;
pub mod shared;
mod slot_element;
mod snippet_block;
mod spread_attribute;
mod style_directive;
mod svelte_body;
mod svelte_boundary;
mod svelte_component;
mod svelte_document;
mod svelte_element;
mod svelte_fragment;
mod svelte_head;
mod svelte_self;
mod svelte_window;
mod text;
mod title_element;
mod transition_directive;
mod use_directive;

pub use animate_directive::visit_animate_directive;
pub use attribute::visit_attribute;
pub use await_block::visit_await_block;
pub use bind_directive::visit_bind_directive;
pub use call_expression::visit_call_expression;
pub use class_directive::visit_class_directive;
pub use component::visit_component;
pub use const_tag::visit_const_tag;
pub use debug_tag::visit_debug_tag;
pub use each_block::visit_each_block;
pub use expression_tag::visit_expression_tag;
pub use html_tag::visit_html_tag;
pub use identifier::visit_identifier;
pub use if_block::visit_if_block;
pub use key_block::visit_key_block;
pub use let_directive::visit_let_directive;
pub use on_directive::visit_on_directive;
pub use regular_element::visit_regular_element;
pub use render_tag::visit_render_tag;
pub use slot_element::visit_slot_element;
pub use snippet_block::visit_snippet_block;
pub use spread_attribute::visit_spread_attribute;
pub use style_directive::visit_style_directive;
pub use svelte_body::visit_svelte_body;
pub use svelte_boundary::visit_svelte_boundary;
pub use svelte_component::visit_svelte_component;
pub use svelte_document::visit_svelte_document;
pub use svelte_element::visit_svelte_element;
pub use svelte_fragment::visit_svelte_fragment;
pub use svelte_head::visit_svelte_head;
pub use svelte_self::visit_svelte_self;
pub use svelte_window::visit_svelte_window;
pub use text::visit_text;
pub use title_element::visit_title_element;
pub use transition_directive::visit_transition_directive;
pub use use_directive::visit_use_directive;
