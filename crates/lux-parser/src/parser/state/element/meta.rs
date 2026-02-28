use crate::context::is_top_level;
use crate::error::{ErrorKind, ParseError};
use crate::input::Input;

const ROOT_ONLY_SVELTE_TAGS: &[&str] = &["head", "options", "window", "document", "body"];

pub(super) fn is_root_only_svelte_tag(kind: &str) -> bool {
    ROOT_ONLY_SVELTE_TAGS.contains(&kind)
}

pub(super) fn enforce_root_only_svelte_tag_rules<'a>(
    input: &mut Input<'a>,
    start: usize,
    name: &'a str,
) {
    let span = oxc_span::Span::new(start as u32, (start + name.len() + 2) as u32);

    if !is_top_level(input) {
        input.state.errors.push(ParseError::with_code(
            ErrorKind::General,
            "svelte_meta_invalid_placement",
            span,
            format!("{name} is only valid at the top level"),
        ));
        return;
    }

    if !input.state.root_meta_tags.insert(name) {
        input.state.errors.push(ParseError::with_code(
            ErrorKind::General,
            "svelte_meta_duplicate",
            span,
            format!("Duplicate root-only Svelte meta tag: {name}"),
        ));
    }
}
