pub mod state;
pub mod read;
pub mod html_entities;
pub mod bracket;
pub mod patterns;
pub mod utils;

use std::collections::HashSet;
use std::sync::LazyLock;

use oxc_allocator::Allocator;
use oxc_ast::ast::{BindingPattern, Expression};
use regex::Regex;
use svelte_ast::css::StyleSheet;
use svelte_ast::node::{AttributeNode, FragmentNode};
use svelte_ast::root::{Fragment, Root, Script, SvelteOptions};
use svelte_ast::span::Span;
use svelte_ast::text::JsComment;

use crate::error::ErrorKind;

/// Stack frame representing a nested element or block being parsed.
/// In JS reference, the stack holds mutable AST nodes directly.
/// In Rust, we store the metadata here and build nodes on pop.
#[derive(Debug)]
pub enum StackFrame<'a> {
    RegularElement {
        start: usize,
        name: String,
        attributes: Vec<AttributeNode<'a>>,
    },
    Component {
        start: usize,
        name: String,
        attributes: Vec<AttributeNode<'a>>,
    },
    SvelteElement {
        start: usize,
        attributes: Vec<AttributeNode<'a>>,
    },
    SvelteComponent {
        start: usize,
        attributes: Vec<AttributeNode<'a>>,
    },
    SvelteSelf {
        start: usize,
        attributes: Vec<AttributeNode<'a>>,
    },
    SvelteHead {
        start: usize,
        attributes: Vec<AttributeNode<'a>>,
    },
    SvelteBody {
        start: usize,
        attributes: Vec<AttributeNode<'a>>,
    },
    SvelteWindow {
        start: usize,
        attributes: Vec<AttributeNode<'a>>,
    },
    SvelteDocument {
        start: usize,
        attributes: Vec<AttributeNode<'a>>,
    },
    SvelteFragment {
        start: usize,
        attributes: Vec<AttributeNode<'a>>,
    },
    SvelteOptions {
        start: usize,
        attributes: Vec<AttributeNode<'a>>,
    },
    TitleElement {
        start: usize,
        attributes: Vec<AttributeNode<'a>>,
    },
    SlotElement {
        start: usize,
        attributes: Vec<AttributeNode<'a>>,
    },
    SvelteBoundary {
        start: usize,
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
        index: Option<String>,
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
        type_params: Option<String>,
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
pub struct LastAutoClosedTag {
    pub tag: String,
    pub reason: String,
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
    pub comments: Vec<JsComment>,

    /// Set of meta tags encountered (for duplicate detection).
    pub meta_tags: HashSet<String>,

    /// Last auto-closed tag info.
    pub last_auto_closed_tag: Option<LastAutoClosedTag>,

    /// Collected errors.
    pub errors: Vec<ParseError>,
}

/// Matches HTML comments (to skip) or `<script` tags with a `lang` attribute.
/// Port of reference's `regex_lang_attribute`.
static REGEX_LANG_ATTRIBUTE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?s)<!--.*?-->|<script\s+(?:[^>]*?\s)?lang=(?:"([^"]*)"|'([^']*)'|([^\s>"']+))[^>]*>"#,
    )
    .unwrap()
});

impl<'a> Parser<'a> {
    /// Create a new parser and run the state machine.
    pub fn new(template: &'a str, allocator: &'a Allocator, loose: bool) -> Self {
        // Reference: `this.template = template.trimEnd()`
        let template = template.trim_end();

        // Detect TypeScript (matches reference constructor logic)
        let ts = detect_lang_ts(template);

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
        };

        // State machine loop (matches reference constructor)
        while parser.index < parser.template.len() {
            state::fragment::fragment(&mut parser);
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
    pub fn into_root(self) -> Root<'a> {
        let fragment_nodes = self.fragments.into_iter().next().unwrap_or_default();
        Root {
            span: Span::new(0, self.template.len()),
            options: self.options,
            fragment: Fragment { nodes: fragment_nodes },
            css: self.css,
            instance: self.instance,
            module: self.module,
            comments: self.comments,
            ts: self.ts,
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
            self.template.as_bytes().get(self.index).copied()
                == s.as_bytes().first().copied()
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
    pub fn eat_required(&mut self, s: &str) {
        if !self.eat(s) {
            if !self.loose {
                self.error(
                    ErrorKind::ExpectedToken,
                    self.index,
                    format!("Expected '{}'", s),
                );
            }
        }
    }

    /// `parser.eat(str, true, required_in_loose)` — with loose control.
    pub fn eat_required_with_loose(&mut self, s: &str, required_in_loose: bool) -> bool {
        if self.eat(s) {
            return true;
        }
        if !self.loose || required_in_loose {
            self.error(
                ErrorKind::ExpectedToken,
                self.index,
                format!("Expected '{}'", s),
            );
        }
        false
    }

    /// `parser.match_regex(pattern)` — match regex at current position.
    /// Returns the matched string if it matches at position 0.
    pub fn match_regex(&self, re: &Regex) -> Option<&'a str> {
        let remaining = &self.template[self.index..];
        if let Some(m) = re.find(remaining) {
            if m.start() == 0 {
                return Some(&self.template[self.index..self.index + m.end()]);
            }
        }
        None
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
    pub fn require_whitespace(&mut self) {
        if self.index < self.template.len() {
            let ch = self.template.as_bytes()[self.index];
            if ch != b' ' && ch != b'\t' && ch != b'\r' && ch != b'\n' {
                self.error(
                    ErrorKind::ExpectedToken,
                    self.index,
                    "Expected whitespace".to_string(),
                );
            }
        }
        self.allow_whitespace();
    }

    /// `parser.read(pattern)` — match regex and advance if matched.
    pub fn read(&mut self, re: &Regex) -> Option<&'a str> {
        if let Some(matched) = self.match_regex(re) {
            self.index += matched.len();
            Some(matched)
        } else {
            None
        }
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

    /// `parser.read_until(pattern)` — consume until regex matches.
    pub fn read_until(&mut self, re: &Regex) -> &'a str {
        if self.index >= self.template.len() {
            if !self.loose {
                self.error(
                    ErrorKind::UnexpectedEof,
                    self.template.len(),
                    "Unexpected end of input".to_string(),
                );
            }
            return "";
        }

        let start = self.index;
        let remaining = &self.template[self.index..];
        if let Some(m) = re.find(remaining) {
            self.index += m.start();
        } else {
            self.index = self.template.len();
        }
        &self.template[start..self.index]
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

    /// Emit a parse error.
    pub fn error(&mut self, kind: ErrorKind, position: usize, message: String) {
        self.errors.push(ParseError {
            kind,
            position,
            message,
        });
    }
}

// ─── Utility Functions ──────────────────────────────────────

/// Detect if template has `<script lang="ts">`.
/// Port of reference's constructor logic using `regex_lang_attribute`.
fn detect_lang_ts(template: &str) -> bool {
    for caps in REGEX_LANG_ATTRIBUTE.captures_iter(template) {
        let full = caps.get(0).unwrap().as_str();
        // Skip HTML comment matches (reference: `match[0][1] !== 's'`)
        if !full.starts_with("<s") {
            continue;
        }
        // First <script> match with lang attr — check if lang value is "ts"
        let lang = caps.get(1)
            .or_else(|| caps.get(2))
            .or_else(|| caps.get(3))
            .map(|m| m.as_str());
        return lang == Some("ts");
    }
    false
}

