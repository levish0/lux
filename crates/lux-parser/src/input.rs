use lux_ast::css::StyleSheet;
use lux_ast::template::root::Script;
use oxc_allocator::Allocator;
use rustc_hash::FxHashSet;
use winnow::stream::{LocatingSlice, Stateful};

use crate::error::{ParseError, ParseWarning};

pub type InputSource<'a> = LocatingSlice<&'a str>;
pub type Input<'a> = Stateful<InputSource<'a>, ParserState<'a>>;

pub struct ParserState<'a> {
    pub allocator: &'a Allocator,
    pub template: &'a str,
    pub ts: bool,
    pub loose: bool,
    pub depth: u32,
    pub shadowroot_depth: u32,
    pub root_meta_tags: FxHashSet<&'a str>,
    pub instance: Option<Script<'a>>,
    pub module: Option<Script<'a>>,
    pub css: Option<StyleSheet<'a>>,
    pub errors: Vec<ParseError>,
    pub warnings: Vec<ParseWarning>,
}

impl<'a> std::fmt::Debug for ParserState<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParserState")
            .field("ts", &self.ts)
            .field("loose", &self.loose)
            .field("depth", &self.depth)
            .field("errors", &self.errors.len())
            .field("warnings", &self.warnings.len())
            .finish()
    }
}

impl<'a> ParserState<'a> {
    pub fn new(allocator: &'a Allocator, template: &'a str, ts: bool, loose: bool) -> Self {
        Self {
            allocator,
            template,
            ts,
            loose,
            depth: 0,
            shadowroot_depth: 0,
            root_meta_tags: FxHashSet::default(),
            instance: None,
            module: None,
            css: None,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
}
