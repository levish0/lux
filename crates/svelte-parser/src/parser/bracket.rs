/// Bracket matching utilities.
/// Direct port of reference utils/bracket.js.

/// Find the corresponding closing bracket, ignoring brackets inside strings, comments, or regex.
/// `index` is the position AFTER the opening bracket.
/// Returns the position of the closing bracket, or None if not found.
pub fn find_matching_bracket(template: &str, index: usize, open: char) -> Option<usize> {
    let close = match open {
        '{' => '}',
        '(' => ')',
        '[' => ']',
        '<' => '>',
        _ => return None,
    };

    let bytes = template.as_bytes();
    let mut brackets = 1u32;
    let mut i = index;

    while brackets > 0 && i < bytes.len() {
        let ch = bytes[i];
        match ch {
            b'\'' | b'"' | b'`' => {
                let end = find_string_end(template, i + 1, ch as char);
                if end >= bytes.len() {
                    return None;
                }
                i = end + 1;
            }
            b'/' => {
                if i + 1 >= bytes.len() {
                    i += 1;
                    continue;
                }
                let next_ch = bytes[i + 1];
                if next_ch == b'/' {
                    // Line comment
                    if let Some(nl) = template[i..].find('\n') {
                        i += nl + 1;
                    } else {
                        i = bytes.len();
                    }
                } else if next_ch == b'*' {
                    // Block comment
                    if let Some(end) = template[i + 2..].find("*/") {
                        i = i + 2 + end + 2;
                    } else {
                        i = bytes.len();
                    }
                } else {
                    // Regex: find unescaped /
                    let end = find_regex_end(template, i + 1);
                    if end >= bytes.len() {
                        i += 1;
                    } else {
                        i = end + 1;
                    }
                }
            }
            _ => {
                if ch == open as u8 {
                    brackets += 1;
                } else if ch == close as u8 {
                    brackets -= 1;
                }
                if brackets == 0 {
                    return Some(i);
                }
                i += 1;
            }
        }
    }

    None
}

/// Match brackets with full stack-based tracking (for `match_bracket` in reference).
/// Starts at `start` (the position of the opening bracket).
/// Returns position AFTER the final closing bracket.
pub fn match_bracket(template: &str, start: usize, brackets: &[(char, char)]) -> Option<usize> {
    let close_chars: Vec<char> = brackets.iter().map(|(_, c)| *c).collect();
    let mut bracket_stack: Vec<char> = Vec::new();
    let bytes = template.as_bytes();
    let mut i = start;

    while i < bytes.len() {
        let ch = bytes[i] as char;
        i += 1;

        if ch == '\'' || ch == '"' || ch == '`' {
            i = match_quote(template, i, ch)?;
            continue;
        }

        if brackets.iter().any(|(o, _)| *o == ch) {
            bracket_stack.push(ch);
        } else if close_chars.contains(&ch) {
            let popped = bracket_stack.pop();
            if let Some(open) = popped {
                let expected = brackets.iter().find(|(o, _)| *o == open).map(|(_, c)| *c);
                if expected != Some(ch) {
                    return None; // mismatched bracket
                }
            }
            if bracket_stack.is_empty() {
                return Some(i);
            }
        }
    }

    None
}

/// Match a quote (string literal), handling escape sequences and template literal interpolation.
/// `start` is position AFTER the opening quote.
/// Returns position AFTER the closing quote.
fn match_quote(template: &str, start: usize, quote: char) -> Option<usize> {
    let bytes = template.as_bytes();
    let mut is_escaped = false;
    let mut i = start;

    while i < bytes.len() {
        let ch = bytes[i] as char;
        i += 1;

        if is_escaped {
            is_escaped = false;
            continue;
        }

        if ch == quote {
            return Some(i);
        }

        if ch == '\\' {
            is_escaped = true;
        }

        if quote == '`' && ch == '$' && i < bytes.len() && bytes[i] == b'{' {
            // Template literal interpolation: ${...}
            i += 1; // skip {
            let default_brackets = [( '{', '}'), ('(', ')'), ('[', ']')];
            // Find matching } using match_bracket logic
            let end = match_bracket(template, i - 1, &default_brackets)?;
            i = end;
        }
    }

    None // unterminated string
}

/// Find the end of a string literal.
/// `start` is position AFTER the opening quote.
/// Returns position of the closing quote character (NOT after it).
fn find_string_end(string: &str, search_start: usize, quote: char) -> usize {
    let search_str = if quote == '`' {
        string
    } else {
        // Non-template strings: only search until newline
        let nl_pos = string[search_start..].find('\n')
            .map(|p| search_start + p)
            .unwrap_or(string.len());
        &string[..nl_pos]
    };
    find_unescaped_char(search_str, search_start, quote as u8)
}

/// Find the end of a regex literal (unescaped /).
fn find_regex_end(string: &str, search_start: usize) -> usize {
    find_unescaped_char(string, search_start, b'/')
}

/// Find the first unescaped instance of `ch`.
fn find_unescaped_char(string: &str, search_start: usize, ch: u8) -> usize {
    let bytes = string.as_bytes();
    let mut i = search_start;
    loop {
        if i >= bytes.len() {
            return usize::MAX;
        }
        if let Some(pos) = bytes[i..].iter().position(|&b| b == ch) {
            let found = i + pos;
            if count_leading_backslashes(bytes, found) % 2 == 0 {
                return found;
            }
            i = found + 1;
        } else {
            return usize::MAX;
        }
    }
}

/// Count consecutive backslashes before a position.
fn count_leading_backslashes(bytes: &[u8], before: usize) -> usize {
    let mut count = 0;
    let mut i = before;
    while i > 0 {
        i -= 1;
        if bytes[i] == b'\\' {
            count += 1;
        } else {
            break;
        }
    }
    count
}
