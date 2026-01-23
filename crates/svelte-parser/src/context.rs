use svelte_ast::css::StyleSheet;
use svelte_ast::root::Script;
use svelte_ast::text::JsComment;

use crate::error::ParseError;

/// Info about an element currently being parsed (for stack tracking)
#[derive(Debug, Clone)]
pub struct ElementStackEntry {
    pub has_shadowrootmode: bool,
}

#[derive(Debug)]
pub struct ParseContext {
    pub ts: bool,
    pub loose: bool,
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
    pub fn new(ts: bool, loose: bool) -> Self {
        Self {
            ts,
            loose,
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
        self.element_stack.push(ElementStackEntry {
            has_shadowrootmode,
        });
    }

    pub fn pop_element(&mut self) {
        self.element_stack.pop();
    }
}
