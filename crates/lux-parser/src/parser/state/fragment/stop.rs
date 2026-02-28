fn peek_tag_name(source: &str) -> Option<&str> {
    let after_open = source.strip_prefix('<')?;
    let end = after_open.find(|ch: char| {
        !ch.is_ascii_alphanumeric() && ch != '-' && ch != '_' && ch != ':' && ch != '.'
    })?;

    if end == 0 {
        return None;
    }

    Some(&after_open[..end])
}

pub fn should_stop_for_element_fragment(source: &str, closing_tag: &str) -> bool {
    if source.starts_with("</") {
        // Any closing tag ends this nested fragment.
        // Caller validates exact closing name and consumes if it matches.
        return true;
    }

    if source.starts_with('<')
        && !source.starts_with("<!")
        && let Some(next_name) = peek_tag_name(source)
    {
        return lux_utils::closing_tag::closing_tag_omitted(closing_tag, Some(next_name));
    }

    false
}

pub fn is_block_delimiter(source: &str) -> bool {
    if let Some(rest) = source.strip_prefix('{') {
        let rest = rest.trim_start();
        return rest.starts_with(':')
            || (rest.starts_with('/') && !rest.starts_with("/*") && !rest.starts_with("//"));
    }

    false
}
