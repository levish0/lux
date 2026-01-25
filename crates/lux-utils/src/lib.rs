//! Utility functions for Svelte/Lux compiler.
//!
//! This module provides various helper functions ported from Svelte's utils.js.

mod attributes;
mod bindings;
mod elements;
mod events;
mod fuzzymatch;
mod hash;
mod reserved;
mod runes;

// Re-export all public items
pub use attributes::*;
pub use bindings::*;
pub use elements::*;
pub use events::*;
pub use fuzzymatch::*;
pub use hash::*;
pub use reserved::*;
pub use runes::*;
