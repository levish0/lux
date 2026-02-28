mod destructuring;
mod pattern;

pub(super) use destructuring::collect_destructuring_expression_bindings;
pub(super) use pattern::{CollectedBinding, collect_pattern_bindings};
