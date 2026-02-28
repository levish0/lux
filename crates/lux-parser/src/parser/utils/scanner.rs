const EXPRESSION_NESTING_PAIRS: &[(u8, u8)] = &[(b'{', b'}'), (b'(', b')'), (b'[', b']')];

#[derive(Clone, Copy)]
enum TopLevelStop<'a> {
    None,
    Expression(&'a [u8]),
    EachAs,
}

pub fn scan_expression_boundary(source: &str, extra_stops: &[u8]) -> Option<usize> {
    scan(
        source,
        0,
        None,
        TopLevelStop::Expression(extra_stops),
        EXPRESSION_NESTING_PAIRS,
    )
}

pub fn scan_each_expression_boundary(source: &str) -> Option<usize> {
    scan(
        source,
        0,
        None,
        TopLevelStop::EachAs,
        EXPRESSION_NESTING_PAIRS,
    )
}

/// Find the byte index of the closing bracket matching `open`.
///
/// `start` must point right after the opening bracket.
pub fn find_matching_bracket(source: &str, start: usize, open: char) -> Option<usize> {
    let close = match open {
        '{' => '}',
        '(' => ')',
        '[' => ']',
        '<' => '>',
        _ => return None,
    };

    if !open.is_ascii() || !close.is_ascii() {
        return None;
    }

    let pair = [(open as u8, close as u8)];
    scan(source, start, Some(close as u8), TopLevelStop::None, &pair)
}

fn scan(
    source: &str,
    mut index: usize,
    terminator: Option<u8>,
    top_level_stop: TopLevelStop<'_>,
    nesting_pairs: &[(u8, u8)],
) -> Option<usize> {
    let bytes = source.as_bytes();

    while index < bytes.len() {
        let byte = bytes[index];

        if let Some(close) = terminator {
            if byte == close {
                return Some(index);
            }
        } else if should_stop_top_level(top_level_stop, source, index, byte) {
            return Some(index);
        }

        index = match byte {
            b'\'' | b'"' => skip_string(bytes, index)?,
            b'`' => skip_template_literal(source, index)?,
            b'/' if index + 1 < bytes.len() && bytes[index + 1] == b'/' => {
                skip_line_comment(bytes, index)
            }
            b'/' if index + 1 < bytes.len() && bytes[index + 1] == b'*' => {
                skip_block_comment(bytes, index)?
            }
            _ => {
                if let Some(close) = close_for_open(nesting_pairs, byte) {
                    scan(
                        source,
                        index + 1,
                        Some(close),
                        top_level_stop,
                        nesting_pairs,
                    )?
                } else {
                    index
                }
            }
        };

        index += 1;
    }

    None
}

fn should_stop_top_level(
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

fn close_for_open(pairs: &[(u8, u8)], open: u8) -> Option<u8> {
    pairs
        .iter()
        .find_map(|(o, c)| if *o == open { Some(*c) } else { None })
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

fn skip_string(bytes: &[u8], start: usize) -> Option<usize> {
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
            b if b == quote => return Some(index),
            _ => {}
        }
        index += 1;
    }

    None
}

fn skip_template_literal(source: &str, start: usize) -> Option<usize> {
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

fn skip_line_comment(bytes: &[u8], start: usize) -> usize {
    let mut index = start + 2;
    while index < bytes.len() {
        if bytes[index] == b'\n' {
            return index;
        }
        index += 1;
    }
    bytes.len().saturating_sub(1)
}

fn skip_block_comment(bytes: &[u8], start: usize) -> Option<usize> {
    let mut index = start + 2;
    while index + 1 < bytes.len() {
        if bytes[index] == b'*' && bytes[index + 1] == b'/' {
            return Some(index + 1);
        }
        index += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_expression_stop_at_equals() {
        assert_eq!(scan_expression_boundary("x = 42}", &[b'=']), Some(2));
    }

    #[test]
    fn test_scan_expression_stop_at_equals_nested() {
        assert_eq!(
            scan_expression_boundary("{ a = 1 } = obj}", &[b'=']),
            Some(10)
        );
    }

    #[test]
    fn test_scan_expression_stop_at_comma() {
        assert_eq!(scan_expression_boundary("a, b)", &[b',', b')']), Some(1));
    }

    #[test]
    fn test_scan_expression_stop_at_paren() {
        assert_eq!(scan_expression_boundary("x)", &[b')']), Some(1));
    }

    #[test]
    fn test_scan_each_expression_end() {
        assert_eq!(scan_each_expression_boundary("items as item}"), Some(6));
    }

    #[test]
    fn test_scan_each_expression_end_no_as() {
        assert_eq!(scan_each_expression_boundary("items}"), Some(5));
    }

    #[test]
    fn test_scan_each_expression_end_complex() {
        assert_eq!(
            scan_each_expression_boundary("items.filter(x => x.ok) as item}"),
            Some(24)
        );
    }

    #[test]
    fn test_find_matching_bracket_simple_braces() {
        assert_eq!(find_matching_bracket("{ a }", 1, '{'), Some(4));
    }

    #[test]
    fn test_find_matching_bracket_nested_braces() {
        assert_eq!(find_matching_bracket("{ { a } }", 1, '{'), Some(8));
    }

    #[test]
    fn test_find_matching_bracket_string_inside() {
        assert_eq!(find_matching_bracket("{ '}' }", 1, '{'), Some(6));
    }

    #[test]
    fn test_find_matching_bracket_template_literal() {
        assert_eq!(find_matching_bracket("{ `}` }", 1, '{'), Some(6));
    }

    #[test]
    fn test_find_matching_bracket_line_comment() {
        assert_eq!(find_matching_bracket("{ // }\n}", 1, '{'), Some(7));
    }

    #[test]
    fn test_find_matching_bracket_block_comment() {
        assert_eq!(find_matching_bracket("{ /* } */ }", 1, '{'), Some(10));
    }

    #[test]
    fn test_find_matching_bracket_unmatched() {
        assert_eq!(find_matching_bracket("{ a", 1, '{'), None);
    }

    #[test]
    fn test_find_matching_bracket_parens() {
        assert_eq!(find_matching_bracket("(a + b)", 1, '('), Some(6));
    }

    #[test]
    fn test_find_matching_bracket_nested_template_expression() {
        assert_eq!(find_matching_bracket("{ `${a}` }", 1, '{'), Some(9));
    }
}
