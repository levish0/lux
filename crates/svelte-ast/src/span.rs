use serde::Serialize;

/// Source position span (byte offsets into the original .svelte source).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn empty() -> Self {
        Self { start: 0, end: 0 }
    }
}
