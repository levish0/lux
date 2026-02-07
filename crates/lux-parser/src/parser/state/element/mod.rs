pub mod attribute;
mod comment;
mod component;
mod element_body;
mod regular;
mod script_style;
mod slot;
mod svelte_body;
mod svelte_boundary;
mod svelte_component;
mod svelte_document;
mod svelte_element;
mod svelte_fragment;
mod svelte_head;
mod svelte_options;
mod svelte_self;
mod svelte_window;
mod title;

use lux_ast::template::root::FragmentNode;
use winnow::Result;
use winnow::error::ContextError;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::{literal, take_while};

use crate::input::Input;
use attribute::is_tag_name_char;

pub fn parse_element<'a>(input: &mut Input<'a>) -> Result<Option<FragmentNode<'a>>> {
    let start = input.current_token_start();

    literal("<").parse_next(input)?;

    let remaining: &str = &input.input;

    if remaining.starts_with("!--") {
        return comment::parse_comment(input, start).map(Some);
    }

    if remaining.starts_with('/') {
        return Err(ContextError::new());
    }

    let name: &str = take_while(1.., is_tag_name_char).parse_next(input)?;

    // Top-level <script> / <style>: read content into ParserState, not fragment.
    // Svelte element.js §4.3 steps 9-11: script/style at root level are stored on Root.
    if (name == "script" || name == "style") && input.state.depth == 0 {
        script_style::parse_script_or_style(input, start, name)?;
        return Ok(None);
    }

    dispatch_element(input, start, name).map(Some)
}

fn dispatch_element<'a>(
    input: &mut Input<'a>,
    start: usize,
    name: &'a str,
) -> Result<FragmentNode<'a>> {
    if let Some(svelte_kind) = name.strip_prefix("svelte:") {
        return match svelte_kind {
            "component" => svelte_component::parse_svelte_component(input, start, name),
            "element" => svelte_element::parse_svelte_element(input, start, name),
            "self" => svelte_self::parse_svelte_self(input, start, name),
            "head" => svelte_head::parse_svelte_head(input, start, name),
            "body" => svelte_body::parse_svelte_body(input, start, name),
            "window" => svelte_window::parse_svelte_window(input, start, name),
            "document" => svelte_document::parse_svelte_document(input, start, name),
            "fragment" => svelte_fragment::parse_svelte_fragment(input, start, name),
            "boundary" => svelte_boundary::parse_svelte_boundary(input, start, name),
            "options" => svelte_options::parse_svelte_options(input, start, name),
            _ => Err(ContextError::new()),
        };
    }

    if is_component_name(name) {
        return component::parse_component(input, start, name);
    }

    if name == "slot" {
        return slot::parse_slot(input, start, name);
    }

    if name == "title" {
        return title::parse_title(input, start, name);
    }

    regular::parse_regular_element(input, start, name)
}

fn is_component_name(name: &str) -> bool {
    name.as_bytes()
        .first()
        .is_some_and(|b| b.is_ascii_uppercase())
        || name.contains('.')
}
