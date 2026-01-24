pub mod bracket;
pub mod html_entities;
pub mod read;
pub mod span_offset;
pub mod state;
pub mod utils;

use std::collections::HashSet;
use oxc_allocator::Allocator;
use oxc_ast::ast::{BindingPattern, Expression};
use svelte_ast::css::StyleSheet;
use svelte_ast::node::{AttributeNode, FragmentNode};
use svelte_ast::root::{Fragment, Root, Script, SvelteOptions};
use svelte_ast::span::{Position, SourceLocation, Span};
use svelte_ast::text::JsComment;
use line_span::LineSpans;
use crate::error::ErrorKind;

/// Stack frame representing a nested element or block being parsed.
/// In JS reference, the stack holds mutable AST nodes directly.
/// In Rust, we store the metadata here and build nodes on pop.
#[derive(Debug)]
pub enum StackFrame<'a> {
    RegularElement {
        start: usize,
        name: &'a str,
        name_loc: SourceLocation,
        attributes: Vec<AttributeNode<'a>>,
    },
    Component {
        start: usize,
        name: &'a str,
        name_loc: SourceLocation,
        attributes: Vec<AttributeNode<'a>>,
    },
    SvelteElement {
        start: usize,
        name: &'a str,
        name_loc: SourceLocation,
        tag: Option<Expression<'a>>,
        attributes: Vec<AttributeNode<'a>>,
    },
    SvelteComponent {
        start: usize,
        name: &'a str,
        name_loc: SourceLocation,
        expression: Option<Expression<'a>>,
        attributes: Vec<AttributeNode<'a>>,
    },
    SvelteSelf {
        start: usize,
        name: &'a str,
        name_loc: SourceLocation,
        attributes: Vec<AttributeNode<'a>>,
    },
    SvelteHead {
        start: usize,
        name: &'a str,
        name_loc: SourceLocation,
        attributes: Vec<AttributeNode<'a>>,
    },
    SvelteBody {
        start: usize,
        name: &'a str,
        name_loc: SourceLocation,
        attributes: Vec<AttributeNode<'a>>,
    },
    SvelteWindow {
        start: usize,
        name: &'a str,
        name_loc: SourceLocation,
        attributes: Vec<AttributeNode<'a>>,
    },
    SvelteDocument {
        start: usize,
        name: &'a str,
        name_loc: SourceLocation,
        attributes: Vec<AttributeNode<'a>>,
    },
    SvelteFragment {
        start: usize,
        name: &'a str,
        name_loc: SourceLocation,
        attributes: Vec<AttributeNode<'a>>,
    },
    SvelteOptions {
        start: usize,
        name: &'a str,
        name_loc: SourceLocation,
        attributes: Vec<AttributeNode<'a>>,
    },
    TitleElement {
        start: usize,
        name: &'a str,
        name_loc: SourceLocation,
        attributes: Vec<AttributeNode<'a>>,
    },
    SlotElement {
        start: usize,
        name: &'a str,
        name_loc: SourceLocation,
        attributes: Vec<AttributeNode<'a>>,
    },
    SvelteBoundary {
        start: usize,
        name: &'a str,
        name_loc: SourceLocation,
        attributes: Vec<AttributeNode<'a>>,
    },
    IfBlock {
        start: usize,
        elseif: bool,
        test: Expression<'a>,
        /// Set when `:else` is encountered — holds the consequent nodes collected so far
        consequent: Option<Vec<FragmentNode<'a>>>,
    },
    EachBlock {
        start: usize,
        expression: Expression<'a>,
        context: Option<BindingPattern<'a>>,
        index: Option<&'a str>,
        key: Option<Expression<'a>>,
        /// Set when `:else` (fallback) is encountered
        body: Option<Vec<FragmentNode<'a>>>,
    },
    AwaitBlock {
        start: usize,
        expression: Expression<'a>,
        value: Option<BindingPattern<'a>>,
        error: Option<BindingPattern<'a>>,
        pending: Option<Vec<FragmentNode<'a>>>,
        then: Option<Vec<FragmentNode<'a>>>,
        phase: AwaitPhase,
    },
    KeyBlock {
        start: usize,
        expression: Expression<'a>,
    },
    SnippetBlock {
        start: usize,
        expression: Expression<'a>,
        parameters: Vec<BindingPattern<'a>>,
        type_params: Option<&'a str>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AwaitPhase {
    Pending,
    Then,
    Catch,
}

impl<'a> StackFrame<'a> {
    pub fn start(&self) -> usize {
        match self {
            Self::RegularElement { start, .. }
            | Self::Component { start, .. }
            | Self::SvelteElement { start, .. }
            | Self::SvelteComponent { start, .. }
            | Self::SvelteSelf { start, .. }
            | Self::SvelteHead { start, .. }
            | Self::SvelteBody { start, .. }
            | Self::SvelteWindow { start, .. }
            | Self::SvelteDocument { start, .. }
            | Self::SvelteFragment { start, .. }
            | Self::SvelteOptions { start, .. }
            | Self::TitleElement { start, .. }
            | Self::SlotElement { start, .. }
            | Self::SvelteBoundary { start, .. }
            | Self::IfBlock { start, .. }
            | Self::EachBlock { start, .. }
            | Self::AwaitBlock { start, .. }
            | Self::KeyBlock { start, .. }
            | Self::SnippetBlock { start, .. } => *start,
        }
    }
}

/// Info about the last auto-closed tag (matches reference)
#[derive(Debug)]
pub struct LastAutoClosedTag<'a> {
    pub tag: &'a str,
    pub reason: &'a str,
    pub depth: usize,
}

/// A parser error.
#[derive(Debug, Clone)]
pub struct ParseError {
    pub kind: ErrorKind,
    pub position: usize,
    pub message: String,
}

/// The Svelte parser. Direct port of the reference parser (index.js).
pub struct Parser<'a> {
    /// The full template source text.
    pub template: &'a str,
    /// Current byte position in the template.
    pub index: usize,
    /// Whether to parse as TypeScript.
    pub ts: bool,
    /// Whether to use loose/error-tolerant parsing.
    pub loose: bool,
    /// The shared OXC allocator for all JS/TS parsing.
    pub allocator: &'a Allocator,

    /// Stack for nested elements/blocks (parallel to `fragments`).
    pub stack: Vec<StackFrame<'a>>,
    /// Stack of fragment node buffers (parallel to `stack`).
    /// `fragments.last()` is always where `append()` pushes.
    pub fragments: Vec<Vec<FragmentNode<'a>>>,

    /// Parsed <script> (instance).
    pub instance: Option<Script<'a>>,
    /// Parsed <script module>.
    pub module: Option<Script<'a>>,
    /// Parsed <style>.
    pub css: Option<StyleSheet<'a>>,
    /// Parsed <svelte:options>.
    pub options: Option<SvelteOptions<'a>>,
    /// Collected JS comments from expressions and scripts.
    pub comments: Vec<JsComment<'a>>,

    /// Set of meta tags encountered (for duplicate detection).
    pub meta_tags: HashSet<&'a str>,

    /// Last auto-closed tag info.
    pub last_auto_closed_tag: Option<LastAutoClosedTag<'a>>,

    /// Collected errors.
    pub errors: Vec<ParseError>,

    /// Precomputed line start offsets for O(log n) offset→line/column lookup.
    line_starts: Vec<usize>,
}


impl<'a> Parser<'a> {
    /// Create a new parser and run the state machine.
    pub fn new(template: &'a str, allocator: &'a Allocator, loose: bool) -> Self {
        // Reference: `this.template = template.trimEnd()`
        let template = template.trim_end();

        // Detect TypeScript (matches reference constructor logic)
        let ts = detect_lang_ts(template);

        // Precompute line start offsets for locate()
        let line_starts: Vec<usize> = template.line_spans().map(|s| s.start()).collect();

        let mut parser = Self {
            template,
            index: 0,
            ts,
            loose,
            allocator,
            stack: Vec::new(),
            fragments: vec![Vec::new()], // root fragment
            instance: None,
            module: None,
            css: None,
            options: None,
            comments: Vec::new(),
            meta_tags: HashSet::new(),
            last_auto_closed_tag: None,
            errors: Vec::new(),
            line_starts,
        };

        // State machine loop (matches reference constructor)
        while parser.index < parser.template.len() {
            if state::fragment::fragment(&mut parser).is_err() {
                break;
            }
        }

        // Check for unclosed blocks/elements
        if !parser.stack.is_empty() {
            let current = parser.stack.last().unwrap();
            if !parser.loose {
                match current {
                    StackFrame::RegularElement { name, start, .. } => {
                        parser.error(
                            ErrorKind::ElementUnclosed,
                            *start,
                            format!("'<{name}>' was left open"),
                        );
                    }
                    _ => {
                        parser.error(
                            ErrorKind::BlockUnclosed,
                            current.start(),
                            "Block was left open".to_string(),
                        );
                    }
                }
            }
        }

        parser
    }

    /// Build the Root AST node from parsed state.
    pub fn into_root(mut self) -> Root<'a> {
        let mut fragment_nodes = self.fragments.into_iter().next().unwrap_or_default();

        // Extract <svelte:options> from fragment and process into structured options.
        // Reference: index.js constructor, after state machine loop.
        let options = if let Some(idx) = fragment_nodes
            .iter()
            .position(|n| matches!(n, FragmentNode::SvelteOptionsRaw(_)))
        {
            let node = fragment_nodes.remove(idx);
            if let FragmentNode::SvelteOptionsRaw(raw) = node {
                Some(read::options::read_options(raw, &mut self.errors, self.allocator))
            } else {
                unreachable!()
            }
        } else {
            self.options
        };

        Root {
            span: Span::new(0, self.template.len()),
            options,
            fragment: Fragment {
                nodes: fragment_nodes,
            },
            css: self.css,
            instance: self.instance,
            module: self.module,
            comments: self.comments,
            ts: self.ts,
        }
    }

    // ─── Locator ─────────────────────────────────────────────────

    /// Convert a byte offset to a Position { line, column, character }.
    /// Line is 1-based, column is 0-based. Uses binary search on precomputed line_starts.
    pub fn locate(&self, offset: usize) -> Position {
        let line_idx = self
            .line_starts
            .binary_search(&offset)
            .unwrap_or_else(|i| i - 1);
        let line_start = self.line_starts[line_idx];
        Position {
            line: line_idx + 1,
            column: offset - line_start,
            character: offset,
        }
    }

    /// Create a SourceLocation from byte offsets.
    pub fn source_location(&self, start: usize, end: usize) -> SourceLocation {
        SourceLocation {
            start: self.locate(start),
            end: self.locate(end),
        }
    }

    // ─── Helper Methods (matching reference 1:1) ─────────────────

    /// `parser.current()` — get current (top) stack frame.
    pub fn current(&self) -> Option<&StackFrame<'a>> {
        self.stack.last()
    }

    /// Mutable access to current stack frame.
    pub fn current_mut(&mut self) -> Option<&mut StackFrame<'a>> {
        self.stack.last_mut()
    }

    /// `parser.match(str)` — check if template at current position starts with str.
    pub fn match_str(&self, s: &str) -> bool {
        if s.len() == 1 {
            self.template.as_bytes().get(self.index).copied() == s.as_bytes().first().copied()
        } else {
            self.template.get(self.index..self.index + s.len()) == Some(s)
        }
    }

    /// `parser.eat(str, required, required_in_loose)` — consume str if it matches.
    pub fn eat(&mut self, s: &str) -> bool {
        if self.match_str(s) {
            self.index += s.len();
            true
        } else {
            false
        }
    }

    /// `parser.eat(str, true)` — consume str or error.
    pub fn eat_required(&mut self, s: &str) -> Result<(), ParseError> {
        if self.eat(s) {
            return Ok(());
        }
        let err = self.error(
            ErrorKind::ExpectedToken,
            self.index,
            format!("Expected '{}'", s),
        );
        if self.loose { Ok(()) } else { Err(err) }
    }

    /// `parser.eat(str, true, required_in_loose)` — with loose control.
    /// Returns Ok(true) if eaten, Ok(false) if not eaten but allowed, Err if fatal.
    pub fn eat_required_with_loose(
        &mut self,
        s: &str,
        required_in_loose: bool,
    ) -> Result<bool, ParseError> {
        if self.eat(s) {
            return Ok(true);
        }
        if self.loose && !required_in_loose {
            return Ok(false);
        }
        let err = self.error(
            ErrorKind::ExpectedToken,
            self.index,
            format!("Expected '{}'", s),
        );
        if self.loose { Ok(false) } else { Err(err) }
    }



    /// `parser.allow_whitespace()` — skip whitespace.
    pub fn allow_whitespace(&mut self) {
        while self.index < self.template.len() {
            let ch = self.template.as_bytes()[self.index];
            if ch == b' ' || ch == b'\t' || ch == b'\r' || ch == b'\n' {
                self.index += 1;
            } else {
                break;
            }
        }
    }

    /// `parser.require_whitespace()` — error if not whitespace, then skip.
    pub fn require_whitespace(&mut self) -> Result<(), ParseError> {
        if self.index < self.template.len() {
            let ch = self.template.as_bytes()[self.index];
            if ch != b' ' && ch != b'\t' && ch != b'\r' && ch != b'\n' {
                let err = self.error(
                    ErrorKind::ExpectedToken,
                    self.index,
                    "Expected whitespace".to_string(),
                );
                if !self.loose {
                    return Err(err);
                }
            }
        }
        self.allow_whitespace();
        Ok(())
    }

    /// `parser.read_identifier()` — read a JS/TS identifier at current position.
    /// Port of reference's `read_identifier()`.
    /// Handles Unicode code points (ID_Start / ID_Continue) and reserved word checking.
    /// Returns (name, start, end).
    pub fn read_identifier(&mut self) -> (&'a str, usize, usize) {
        let start = self.index;

        if self.index >= self.template.len() {
            return ("", start, start);
        }

        let remaining = &self.template[self.index..];
        let mut chars = remaining.char_indices();

        // First character must be identifier start
        let Some((_, first_char)) = chars.next() else {
            return ("", start, start);
        };

        if !oxc_syntax::identifier::is_identifier_start(first_char) {
            return ("", start, start);
        }

        let mut end = self.index + first_char.len_utf8();

        // Continue characters
        for (byte_offset, ch) in chars {
            if !oxc_syntax::identifier::is_identifier_part(ch) {
                break;
            }
            end = self.index + byte_offset + ch.len_utf8();
        }

        let name = &self.template[start..end];
        self.index = end;

        // Check reserved words (matches reference's is_reserved check)
        if utils::is_reserved(name) && !self.loose {
            self.error(
                ErrorKind::ExpectedToken,
                start,
                format!("'{}' is a reserved word", name),
            );
        }

        (name, start, end)
    }

    /// Fast `read_until` using a byte predicate instead of regex.
    /// Scans from current index until `pred(byte)` returns true.
    /// Returns the consumed slice (does NOT consume the matching byte).
    #[inline]
    pub fn read_until_char(&mut self, pred: impl Fn(u8) -> bool) -> &'a str {
        let start = self.index;
        let bytes = self.template.as_bytes();
        while self.index < bytes.len() {
            if pred(bytes[self.index]) {
                break;
            }
            self.index += 1;
        }
        &self.template[start..self.index]
    }

    /// Fast `read_until` for a literal substring (e.g. `"-->"`, `"</script"`).
    /// Returns the slice before the match. Does NOT consume the matched substring.
    #[inline]
    pub fn read_until_str(&mut self, needle: &str) -> &'a str {
        let start = self.index;
        if let Some(pos) = self.template[self.index..].find(needle) {
            self.index += pos;
        } else {
            self.index = self.template.len();
        }
        &self.template[start..self.index]
    }

    /// Check if the current byte matches a predicate (without consuming).
    #[inline]
    pub fn match_ch(&self, pred: impl Fn(u8) -> bool) -> bool {
        self.template
            .as_bytes()
            .get(self.index)
            .copied()
            .map_or(false, &pred)
    }

    /// Skip whitespace at current position, then check if byte `b` follows.
    /// If yes, consume through the byte and return true. Otherwise restore position.
    #[inline]
    pub fn eat_whitespace_then(&mut self, b: u8) -> bool {
        let saved = self.index;
        self.allow_whitespace();
        if self.template.as_bytes().get(self.index).copied() == Some(b) {
            self.index += 1;
            true
        } else {
            self.index = saved;
            false
        }
    }

    /// Check if remaining input matches `\s*<byte>` without consuming.
    #[inline]
    pub fn peek_whitespace_then(&self, b: u8) -> bool {
        let bytes = self.template.as_bytes();
        let mut i = self.index;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        bytes.get(i).copied() == Some(b)
    }

    /// Read and consume a closing tag like `</script\s*>` or `</style\s*>` at current position.
    /// Returns the consumed slice if matched, None otherwise.
    pub fn eat_closing_tag(&mut self, tag_name: &str) -> Option<&'a str> {
        let start = self.index;
        // Check for `</`
        if !self.match_str("</") {
            return None;
        }
        self.index += 2;
        // Check tag name (case-sensitive)
        if !self.template[self.index..].starts_with(tag_name) {
            self.index = start;
            return None;
        }
        self.index += tag_name.len();
        // Skip optional whitespace
        self.allow_whitespace();
        // Expect `>`
        if self.template.as_bytes().get(self.index).copied() != Some(b'>') {
            self.index = start;
            return None;
        }
        self.index += 1;
        Some(&self.template[start..self.index])
    }

    /// Find the position of a closing tag like `</script\s*>` from the current index.
    /// Advances index to the start of the closing tag. Returns the content before it.
    pub fn read_until_closing_tag(&mut self, tag_name: &str) -> &'a str {
        let start = self.index;
        let needle = format!("</{}", tag_name);
        loop {
            if let Some(pos) = self.template[self.index..].find(&needle) {
                self.index += pos;
                // Verify it's actually followed by optional whitespace then `>`
                let saved = self.index;
                self.index += needle.len();
                self.allow_whitespace();
                if self.template.as_bytes().get(self.index).copied() == Some(b'>') {
                    // Found valid closing tag — restore to start of it
                    self.index = saved;
                    return &self.template[start..saved];
                }
                // Not a valid closing tag, keep scanning
                self.index = saved + needle.len();
            } else {
                self.index = self.template.len();
                return &self.template[start..self.index];
            }
        }
    }

    /// `parser.pop()` — pop both stack and fragments.
    pub fn pop(&mut self) -> (Option<StackFrame<'a>>, Option<Vec<FragmentNode<'a>>>) {
        let fragment = self.fragments.pop();
        let frame = self.stack.pop();
        (frame, fragment)
    }

    /// `parser.append(node)` — add node to current (topmost) fragment.
    pub fn append(&mut self, node: FragmentNode<'a>) {
        if let Some(frag) = self.fragments.last_mut() {
            frag.push(node);
        }
    }

    /// Get the current byte, or None if at end.
    pub fn current_byte(&self) -> Option<u8> {
        self.template.as_bytes().get(self.index).copied()
    }

    /// Get remaining template from current position.
    pub fn remaining(&self) -> &'a str {
        &self.template[self.index..]
    }

    /// Emit a parse error. Returns the error for use with `Err(...)`.
    pub fn error(&mut self, kind: ErrorKind, position: usize, message: String) -> ParseError {
        let err = ParseError {
            kind,
            position,
            message,
        };
        self.errors.push(err.clone());
        err
    }
}

// ─── Utility Functions ──────────────────────────────────────

/// Detect if template has `<script lang="ts">`.
/// Port of reference's constructor logic using `regex_lang_attribute`.
/// Scans for the first `<script` tag (skipping HTML comments) and checks its `lang` attribute.
fn detect_lang_ts(template: &str) -> bool {
    let bytes = template.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Skip HTML comments
        if i + 3 < bytes.len() && &bytes[i..i + 4] == b"<!--" {
            if let Some(end) = template[i + 4..].find("-->") {
                i += 4 + end + 3;
                continue;
            } else {
                break; // unclosed comment
            }
        }
        // Check for `<script` followed by whitespace or `>`
        if i + 7 <= bytes.len()
            && bytes[i] == b'<'
            && template[i + 1..].starts_with("script")
            && (bytes.get(i + 7).map_or(true, |&b| b.is_ascii_whitespace() || b == b'>'))
        {
            // Found <script — scan attributes for `lang=`
            let tag_start = i + 7;
            return find_lang_value(template, tag_start) == Some("ts");
        }
        i += 1;
    }
    false
}

/// Within a `<script ...>` tag's attribute area (starting after `<script`),
/// find the value of the `lang` attribute.
fn find_lang_value(template: &str, start: usize) -> Option<&str> {
    let bytes = template.as_bytes();
    let mut i = start;
    while i < bytes.len() && bytes[i] != b'>' {
        // Skip whitespace
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        // Read attribute name
        let name_start = i;
        while i < bytes.len() && bytes[i] != b'=' && bytes[i] != b'>' && !bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let name = &template[name_start..i];
        // Skip whitespace around `=`
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            // Read value
            let value = if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
                let quote = bytes[i];
                i += 1;
                let val_start = i;
                while i < bytes.len() && bytes[i] != quote {
                    i += 1;
                }
                let val = &template[val_start..i];
                if i < bytes.len() {
                    i += 1; // skip closing quote
                }
                val
            } else {
                // Unquoted value
                let val_start = i;
                while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'>' {
                    i += 1;
                }
                &template[val_start..i]
            };
            if name == "lang" {
                return Some(value);
            }
        } else {
            // Boolean attribute (no value)
            if name == "lang" {
                return Some("");
            }
        }
    }
    None
}
