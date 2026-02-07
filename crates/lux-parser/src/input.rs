use lux_ast::css::StyleSheet;
use lux_ast::template::root::Script;
use oxc_allocator::Allocator;
use winnow::stream::{LocatingSlice, Stateful};

use crate::error::ParseError;

pub type InputSource<'a> = LocatingSlice<&'a str>;
pub type Input<'a> = Stateful<InputSource<'a>, ParserState<'a>>;

pub struct ParserState<'a> {
    pub allocator: &'a Allocator,
    pub template: &'a str,
    pub ts: bool,
    pub depth: u32,
    pub instance: Option<Script<'a>>,
    pub module: Option<Script<'a>>,
    pub css: Option<StyleSheet<'a>>,
    pub errors: Vec<ParseError>,
}

impl<'a> std::fmt::Debug for ParserState<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParserState")
            .field("ts", &self.ts)
            .field("depth", &self.depth)
            .field("errors", &self.errors.len())
            .finish()
    }
}

impl<'a> ParserState<'a> {
    pub fn new(allocator: &'a Allocator, template: &'a str, ts: bool) -> Self {
        Self {
            allocator,
            template,
            ts,
            depth: 0,
            instance: None,
            module: None,
            css: None,
            errors: Vec::new(),
        }
    }
}
