pub mod read;
pub mod state;
pub mod utils;

use std::mem;

use lux_ast::template::root::{FragmentNode, Root};
use oxc_allocator::Allocator;
use oxc_span::Span;
use winnow::stream::{LocatingSlice, Location as StreamLocation, Stateful};

use crate::error::{ErrorKind, ParseError};
use crate::input::ParserState;
use crate::parser::read::options::process_svelte_options;
use crate::parser::state::fragment::parse_fragment;
use crate::parser::utils::language::detect_typescript_lang;

pub struct ParseResult<'a> {
    pub root: Root<'a>,
    pub errors: Vec<ParseError>,
}

pub fn parse<'a>(template: &'a str, allocator: &'a Allocator, ts: bool) -> ParseResult<'a> {
    let effective_ts = ts || detect_typescript_lang(template);
    let state = ParserState::new(allocator, template, effective_ts);
    let mut input = Stateful {
        input: LocatingSlice::new(template),
        state,
    };

    let mut fragment = match parse_fragment(&mut input) {
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
    let mut errors = input.state.errors;

    // Extract all SvelteOptionsRaw nodes from fragment to Root.options.
    let mut options = None;
    let mut options_nodes = Vec::new();
    let nodes = mem::take(&mut fragment.nodes);
    for node in nodes {
        match node {
            FragmentNode::SvelteOptionsRaw(raw) => options_nodes.push(raw),
            other => fragment.nodes.push(other),
        }
    }

    if let Some(first) = options_nodes.into_iter().next() {
        match process_svelte_options(first) {
            Ok(opts) => options = Some(opts),
            Err(e) => errors.push(e),
        }
    }

    let root = Root {
        span: Span::new(0, template.len() as u32),
        options,
        fragment,
        css,
        instance,
        module,
        comments: Vec::new(),
        ts: effective_ts,
    };

    ParseResult { root, errors }
}
