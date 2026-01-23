use std::collections::HashSet;

use svelte_ast::css::StyleSheet;
use svelte_ast::root::Script;
use svelte_ast::text::JsComment;

use crate::error::ParseError;

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
}
