//! Shared utilities for analysis visitors.
//!
//! This module mirrors the structure of:
//! `packages/svelte/src/compiler/phases/2-analyze/visitors/shared/`

mod attribute;
mod component;
mod element;
mod fragment;
mod special_element;
pub mod utils;

pub use attribute::*;
pub use component::*;
pub use element::*;
pub use fragment::*;
pub use special_element::*;
pub use utils::*;
