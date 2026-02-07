pub mod read;
pub mod state;
pub mod utils;

use lux_ast::template::root::Root;
use oxc_allocator::Allocator;
use oxc_span::Span;
use winnow::stream::{LocatingSlice, Stateful};

use crate::error::ParseError;
use crate::input::ParserState;
use crate::parser::state::fragment::parse_fragment;

pub struct ParseResult<'a> {
    pub root: Root<'a>,
    pub errors: Vec<ParseError>,
}

pub fn parse<'a>(template: &'a str, allocator: &'a Allocator, ts: bool) -> ParseResult<'a> {
    let state = ParserState::new(allocator, template, ts);
    let mut input = Stateful {
        input: LocatingSlice::new(template),
        state,
    };

    let fragment = match parse_fragment(&mut input) {
        Ok(f) => f,
        Err(_) => lux_ast::template::root::Fragment {
            nodes: Vec::new(),
            transparent: false,
            dynamic: false,
        },
    };

    let errors = input.state.errors;

    let root = Root {
        span: Span::new(0, template.len() as u32),
        options: None,
        fragment,
        css: None,
        instance: None,
        module: None,
        comments: Vec::new(),
        ts,
    };

    ParseResult { root, errors }
}
