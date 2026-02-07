pub mod read;
pub mod state;
pub mod utils;

use lux_ast::template::root::Root;
use oxc_allocator::Allocator;
use oxc_span::Span;
use winnow::stream::{LocatingSlice, Location as StreamLocation, Stateful};

use crate::error::{ErrorKind, ParseError};
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
        Err(e) => {
            let pos = input.current_token_start();
            input.state.errors.push(ParseError::new(
                ErrorKind::General,
                Span::new(pos as u32, pos as u32),
                format!("Parse error at position {}: {}", pos, e),
            ));
            lux_ast::template::root::Fragment {
                nodes: Vec::new(),
                transparent: false,
                dynamic: false,
            }
        }
    };

    let instance = input.state.instance.take();
    let module = input.state.module.take();
    let css = input.state.css.take();
    let errors = input.state.errors;

    let root = Root {
        span: Span::new(0, template.len() as u32),
        options: None,
        fragment,
        css,
        instance,
        module,
        comments: Vec::new(),
        ts,
    };

    ParseResult { root, errors }
}
