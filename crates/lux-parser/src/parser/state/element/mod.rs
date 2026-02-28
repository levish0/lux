pub mod attribute;
mod comment;
mod component;
mod dispatch;
mod element_body;
mod meta;
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
use crate::input::Input;
use attribute::is_tag_name_char;
use dispatch::dispatch_element;

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
