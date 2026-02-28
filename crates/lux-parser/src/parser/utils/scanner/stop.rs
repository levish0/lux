use super::TopLevelStop;

pub(super) fn should_stop_top_level(
    top_level_stop: TopLevelStop<'_>,
    source: &str,
    index: usize,
    byte: u8,
) -> bool {
    match top_level_stop {
        TopLevelStop::None => false,
        TopLevelStop::Expression(extra_stops) => byte == b'}' || extra_stops.contains(&byte),
        TopLevelStop::EachAs => byte == b'}' || starts_with_each_as(source, index),
    }
}

pub(super) fn close_for_open(pairs: &[(u8, u8)], open: u8) -> Option<u8> {
    pairs.iter().find_map(|(expected_open, close)| {
        if *expected_open == open {
            Some(*close)
        } else {
            None
        }
    })
}

fn starts_with_each_as(source: &str, index: usize) -> bool {
    let bytes = source.as_bytes();

    if bytes[index] != b'a' || index + 1 >= bytes.len() || bytes[index + 1] != b's' {
        return false;
    }

    if index == 0 || !bytes[index - 1].is_ascii_whitespace() {
        return false;
    }

    let next = index + 2;
    if next < bytes.len() {
        let next_byte = bytes[next];
        if next_byte.is_ascii_alphanumeric() || next_byte == b'_' || next_byte == b'$' {
            return false;
        }
    }

    true
}
