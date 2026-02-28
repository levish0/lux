use lux_ast::template::element::SvelteOptionsRaw;
use lux_ast::template::root::SvelteOptions;
use oxc_allocator::Allocator;

use crate::error::{ErrorKind, ParseError};

mod apply;
mod attributes;
mod custom_element;
mod static_value;

use apply::apply_svelte_option;
use attributes::extract_static_attributes;

pub fn process_svelte_options<'a>(
    raw: SvelteOptionsRaw<'a>,
    allocator: &'a Allocator,
) -> Result<SvelteOptions<'a>, ParseError> {
    let span = raw.span;

    if !raw.fragment.nodes.is_empty() {
        return Err(ParseError::with_code(
            ErrorKind::InvalidSvelteOptions,
            "svelte_meta_invalid_content",
            span,
            "svelte:options cannot have child nodes",
        ));
    }

    let attributes = extract_static_attributes(raw.attributes, span)?;

    let mut options = SvelteOptions {
        span,
        runes: None,
        immutable: None,
        accessors: None,
        preserve_whitespace: None,
        namespace: None,
        css: None,
        custom_element: None,
        attributes: Vec::new(),
    };

    for attr in &attributes {
        apply_svelte_option(&mut options, attr, allocator)?;
    }

    options.attributes = attributes;
    Ok(options)
}
