/// Bracket matching for expression boundary detection.
///
/// Tracks (), [], {}, strings, template literals, comments, and regex
/// to find the matching close bracket.

/// Find the position of the matching close bracket.
///
/// `start` should be the position right after the open bracket.
/// Returns the byte index of the matching close bracket, or None.
pub fn find_matching_bracket(input: &str, start: usize, open: char) -> Option<usize> {
    let close = match open {
        '{' => '}',
        '(' => ')',
        '[' => ']',
        _ => return None,
    };

    let bytes = input.as_bytes();
    let mut depth: u32 = 1;
    let mut i = start;

    while i < bytes.len() {
        let b = bytes[i];
        match b {
            b if b == open as u8 => depth += 1,
            b if b == close as u8 => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            b'\'' | b'"' => {
                i = skip_string(bytes, i)?;
            }
            b'`' => {
                i = skip_template_literal(bytes, i)?;
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                i = skip_line_comment(bytes, i);
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                i = skip_block_comment(bytes, i)?;
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Skip a single-quoted or double-quoted string. Returns position of closing quote.
fn skip_string(bytes: &[u8], start: usize) -> Option<usize> {
    let quote = bytes[start];
    let mut i = start + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 1, // skip escaped char
            b if b == quote => return Some(i),
            b'\n' if quote != b'`' => return Some(i), // unterminated, stop at newline
            _ => {}
        }
        i += 1;
    }
    None
}

/// Skip a template literal (backtick). Handles nested ${}.
fn skip_template_literal(bytes: &[u8], start: usize) -> Option<usize> {
    let mut i = start + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 1,
            b'`' => return Some(i),
            b'$' if i + 1 < bytes.len() && bytes[i + 1] == b'{' => {
                i += 2;
                // Find matching } for the template expression
                if let Some(end) = find_matching_bracket_bytes(bytes, i, b'{', b'}') {
                    i = end;
                } else {
                    return None;
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Simple bracket matching on raw bytes.
fn find_matching_bracket_bytes(bytes: &[u8], start: usize, open: u8, close: u8) -> Option<usize> {
    let mut depth: u32 = 1;
    let mut i = start;
    while i < bytes.len() {
        match bytes[i] {
            b if b == open => depth += 1,
            b if b == close => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            b'\'' | b'"' => {
                i = skip_string(bytes, i)?;
            }
            b'`' => {
                i = skip_template_literal(bytes, i)?;
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Skip a // line comment. Returns position of newline (or end of input).
fn skip_line_comment(bytes: &[u8], start: usize) -> usize {
    let mut i = start + 2;
    while i < bytes.len() {
        if bytes[i] == b'\n' {
            return i;
        }
        i += 1;
    }
    bytes.len().saturating_sub(1)
}

/// Skip a /* block comment */. Returns position of closing `*/`.
fn skip_block_comment(bytes: &[u8], start: usize) -> Option<usize> {
    let mut i = start + 2;
    while i + 1 < bytes.len() {
        if bytes[i] == b'*' && bytes[i + 1] == b'/' {
            return Some(i + 1);
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_braces() {
        assert_eq!(find_matching_bracket("{ a }", 1, '{'), Some(4));
    }

    #[test]
    fn test_nested_braces() {
        assert_eq!(find_matching_bracket("{ { a } }", 1, '{'), Some(8));
    }

    #[test]
    fn test_string_inside() {
        assert_eq!(find_matching_bracket("{ '}' }", 1, '{'), Some(6));
    }

    #[test]
    fn test_template_literal() {
        assert_eq!(find_matching_bracket("{ `}` }", 1, '{'), Some(6));
    }

    #[test]
    fn test_line_comment() {
        assert_eq!(find_matching_bracket("{ // }\n}", 1, '{'), Some(7));
    }

    #[test]
    fn test_block_comment() {
        assert_eq!(find_matching_bracket("{ /* } */ }", 1, '{'), Some(10));
    }

    #[test]
    fn test_unmatched() {
        assert_eq!(find_matching_bracket("{ a", 1, '{'), None);
    }

    #[test]
    fn test_parens() {
        assert_eq!(find_matching_bracket("(a + b)", 1, '('), Some(6));
    }

    #[test]
    fn test_nested_template_expression() {
        assert_eq!(find_matching_bracket("{ `${a}` }", 1, '{'), Some(9));
    }
}
