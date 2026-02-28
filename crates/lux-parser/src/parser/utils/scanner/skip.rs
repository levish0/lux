use super::{EXPRESSION_NESTING_PAIRS, TopLevelStop, scan};

pub(super) fn skip_string(bytes: &[u8], start: usize) -> Option<usize> {
    let quote = bytes[start];
    let mut index = start + 1;

    while index < bytes.len() {
        match bytes[index] {
            b'\\' => {
                if index + 1 >= bytes.len() {
                    return None;
                }
                index += 2;
                continue;
            }
            byte if byte == quote => return Some(index),
            _ => {}
        }
        index += 1;
    }

    None
}

pub(super) fn skip_template_literal(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut index = start + 1;

    while index < bytes.len() {
        match bytes[index] {
            b'\\' => {
                if index + 1 >= bytes.len() {
                    return None;
                }
                index += 2;
                continue;
            }
            b'`' => return Some(index),
            b'$' if index + 1 < bytes.len() && bytes[index + 1] == b'{' => {
                let close = scan(
                    source,
                    index + 2,
                    Some(b'}'),
                    TopLevelStop::None,
                    EXPRESSION_NESTING_PAIRS,
                )?;
                index = close + 1;
                continue;
            }
            _ => {}
        }
        index += 1;
    }

    None
}

pub(super) fn skip_line_comment(bytes: &[u8], start: usize) -> usize {
    let mut index = start + 2;
    while index < bytes.len() {
        if bytes[index] == b'\n' {
            return index;
        }
        index += 1;
    }
    bytes.len().saturating_sub(1)
}

pub(super) fn skip_block_comment(bytes: &[u8], start: usize) -> Option<usize> {
    let mut index = start + 2;
    while index + 1 < bytes.len() {
        if bytes[index] == b'*' && bytes[index + 1] == b'/' {
            return Some(index + 1);
        }
        index += 1;
    }
    None
}
