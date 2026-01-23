use std::collections::HashSet;

use svelte_ast::css::StyleSheet;
use svelte_ast::root::Script;
use svelte_ast::text::JsComment;

use crate::error::ParseError;

/// Info about an element currently being parsed (for stack tracking)
#[derive(Debug, Clone)]
pub struct ElementStackEntry {
    pub name: String,
    pub has_shadowrootmode: bool,
}

#[derive(Debug)]
pub struct ParseContext {
    pub ts: bool,
    pub loose: bool,
    pub recursion_depth: usize,
    pub max_recursion_depth: usize,
    pub meta_tags: HashSet<&'static str>,
    pub comments: Vec<JsComment>,
    pub errors: Vec<ParseError>,
    pub instance: Option<Script>,
    pub module: Option<Script>,
    pub css: Option<StyleSheet>,
    /// Stack of elements currently being parsed
    pub element_stack: Vec<ElementStackEntry>,
}

impl ParseContext {
    pub fn new(ts: bool, loose: bool) -> Self {
        Self {
            ts,
            loose,
            recursion_depth: 0,
            max_recursion_depth: 128,
            meta_tags: HashSet::new(),
            comments: Vec::new(),
            errors: Vec::new(),
            instance: None,
            module: None,
            css: None,
            element_stack: Vec::new(),
        }
    }

    pub fn increase_depth(&mut self) -> bool {
        if self.recursion_depth >= self.max_recursion_depth {
            return false;
        }
        self.recursion_depth += 1;
        true
    }

    pub fn decrease_depth(&mut self) {
        self.recursion_depth = self.recursion_depth.saturating_sub(1);
    }

    /// Check if any parent element has shadowrootmode attribute
    /// (for deciding if <slot> should be SlotElement or RegularElement)
    pub fn parent_is_shadowroot_template(&self) -> bool {
        self.element_stack
            .iter()
            .any(|entry| entry.has_shadowrootmode)
    }

    pub fn push_element(&mut self, name: String, has_shadowrootmode: bool) {
        self.element_stack.push(ElementStackEntry {
            name,
            has_shadowrootmode,
        });
    }

    pub fn pop_element(&mut self) {
        self.element_stack.pop();
    }
}
