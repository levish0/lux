use oxc_allocator::Allocator;
use winnow::stream::{LocatingSlice, Stateful};

use crate::error::ParseError;

pub type InputSource<'a> = LocatingSlice<&'a str>;
pub type Input<'a> = Stateful<InputSource<'a>, ParserState<'a>>;

pub struct ParserState<'a> {
    pub allocator: &'a Allocator,
    pub ts: bool,
    pub errors: Vec<ParseError>,
}

impl<'a> std::fmt::Debug for ParserState<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParserState")
            .field("ts", &self.ts)
            .field("errors", &self.errors.len())
            .finish()
    }
}

impl<'a> ParserState<'a> {
    pub fn new(allocator: &'a Allocator, ts: bool) -> Self {
        Self {
            allocator,
            ts,
            errors: Vec::new(),
        }
    }
}
