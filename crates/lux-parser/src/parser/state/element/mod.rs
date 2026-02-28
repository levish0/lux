pub mod attribute;
mod comment;
mod component;
mod element_body;
mod regular;
mod script_style;
mod slot;
mod svelte;
mod title;

use lux_ast::template::root::FragmentNode;
use winnow::Result;
use winnow::error::ContextError;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::{literal, take_while};

use crate::context::is_top_level;
use crate::error::{ErrorKind, ParseError};
use crate::input::Input;
use attribute::is_tag_name_char;

const ROOT_ONLY_SVELTE_TAGS: &[&str] = &["head", "options", "window", "document", "body"];

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

    if (name == "script" || name == "style") && is_top_level(input) {
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
        if ROOT_ONLY_SVELTE_TAGS.contains(&svelte_kind) {
            enforce_root_only_svelte_tag_rules(input, start, name);
        }

        return match svelte_kind {
            "component" => svelte::svelte_component::parse_svelte_component(input, start, name),
            "element" => svelte::svelte_element::parse_svelte_element(input, start, name),
            "self" => svelte::svelte_self::parse_svelte_self(input, start, name),
            "head" => svelte::svelte_head::parse_svelte_head(input, start, name),
            "body" => svelte::svelte_body::parse_svelte_body(input, start, name),
            "window" => svelte::svelte_window::parse_svelte_window(input, start, name),
            "document" => svelte::svelte_document::parse_svelte_document(input, start, name),
            "fragment" => svelte::svelte_fragment::parse_svelte_fragment(input, start, name),
            "boundary" => svelte::svelte_boundary::parse_svelte_boundary(input, start, name),
            "options" => svelte::svelte_options::parse_svelte_options(input, start, name),
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

fn enforce_root_only_svelte_tag_rules<'a>(input: &mut Input<'a>, start: usize, name: &'a str) {
    let span = oxc_span::Span::new(start as u32, (start + name.len() + 2) as u32);

    if !is_top_level(input) {
        input.state.errors.push(ParseError::new(
            ErrorKind::General,
            span,
            format!("{name} is only valid at the top level"),
        ));
        return;
    }

    if !input.state.root_meta_tags.insert(name) {
        input.state.errors.push(ParseError::new(
            ErrorKind::General,
            span,
            format!("Duplicate root-only Svelte meta tag: {name}"),
        ));
    }
}
