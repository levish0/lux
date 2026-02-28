use lux_ast::template::root::FragmentNode;
use winnow::Result;
use winnow::error::ContextError;

use crate::context::in_shadowroot_template;
use crate::input::Input;

use super::meta::{enforce_root_only_svelte_tag_rules, is_root_only_svelte_tag};
use super::{component, regular, slot, svelte, title};

pub(super) fn dispatch_element<'a>(
    input: &mut Input<'a>,
    start: usize,
    name: &'a str,
) -> Result<FragmentNode<'a>> {
    if let Some(svelte_kind) = name.strip_prefix("svelte:") {
        if is_root_only_svelte_tag(svelte_kind) {
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

    if name == "slot" && !in_shadowroot_template(input) {
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
