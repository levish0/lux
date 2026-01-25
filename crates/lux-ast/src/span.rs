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

/// A position in source code with line, column, and character offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub character: usize,
}

/// A source location range with start and end positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct SourceLocation {
    pub start: Position,
    pub end: Position,
}
