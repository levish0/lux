use line_span::LineSpanExt;
use svelte_ast::css::StyleSheet;
use svelte_ast::root::Script;
use svelte_ast::span::{Position, SourceLocation};
use svelte_ast::text::JsComment;

use crate::error::ParseError;

/// Converts byte offsets to line/column positions using binary search.
#[derive(Debug)]
pub struct Locator {
    /// Byte offset of the start of each line (sorted).
    line_starts: Vec<usize>,
}

impl Locator {
    /// Build the locator from source using `line_span`.
    pub fn new(source: &str) -> Self {
        let line_starts: Vec<usize> = source.line_spans().map(|s| s.range().start).collect();
        Self { line_starts }
    }

    /// Convert a byte offset to a Position { line (1-based), column (0-based), character }.
    pub fn locate(&self, offset: usize) -> Position {
        let line_idx = match self.line_starts.binary_search(&offset) {
            Ok(idx) => idx,
            Err(idx) => idx - 1,
        };
        Position {
            line: line_idx + 1,
            column: offset - self.line_starts[line_idx],
            character: offset,
        }
    }

    /// Convert byte offsets to a SourceLocation.
    pub fn locate_span(&self, start: usize, end: usize) -> SourceLocation {
        SourceLocation {
            start: self.locate(start),
            end: self.locate(end),
        }
    }
}

/// Info about an element currently being parsed (for stack tracking)
#[derive(Debug, Clone)]
pub struct ElementStackEntry {
    pub has_shadowrootmode: bool,
}

#[derive(Debug)]
pub struct ParseContext {
    pub ts: bool,
    pub loose: bool,
    pub locator: Locator,
    pub comments: Vec<JsComment>,
    pub errors: Vec<ParseError>,
    pub instance: Option<Script>,
    pub module: Option<Script>,
    pub css: Option<StyleSheet>,
    /// Stack of elements currently being parsed
    pub element_stack: Vec<ElementStackEntry>,
    /// When true, quoted attribute values are parsed as plain text (no expression interpolation)
    pub text_only_attributes: bool,
}

impl ParseContext {
    pub fn new(source: &str, ts: bool, loose: bool) -> Self {
        Self {
            ts,
            loose,
            locator: Locator::new(source),
            comments: Vec::new(),
            errors: Vec::new(),
            instance: None,
            module: None,
            css: None,
            element_stack: Vec::new(),
            text_only_attributes: false,
        }
    }

    /// Check if any parent element has shadowrootmode attribute
    /// (for deciding if <slot> should be SlotElement or RegularElement)
    pub fn parent_is_shadowroot_template(&self) -> bool {
        self.element_stack
            .iter()
            .any(|entry| entry.has_shadowrootmode)
    }

    pub fn push_element(&mut self, has_shadowrootmode: bool) {
        self.element_stack
            .push(ElementStackEntry { has_shadowrootmode });
    }

    pub fn pop_element(&mut self) {
        self.element_stack.pop();
    }
}
