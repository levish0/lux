use winnow::Result as ParseResult;
use winnow::prelude::*;
use winnow::token::take;

use super::ParserInput;

/// Find the byte offset of the matching closing bracket in `s`.
///
/// `s` should start AFTER the opening bracket.
/// Returns the byte offset of the matching closing bracket (relative to start of `s`),
/// or `None` if not found.
///
/// Handles: nested brackets, string literals ('/"/ `), template literals with ${},
/// line comments (//), block comments (/* */).
pub fn find_matching_bracket(s: &str, open: char) -> Option<usize> {
    let close = match open {
        '{' => '}',
        '(' => ')',
        '[' => ']',
        _ => return None,
    };

    let bytes = s.as_bytes();
    let mut depth: u32 = 1;
    let mut i = 0;

    while i < bytes.len() && depth > 0 {
        let ch = bytes[i];
        match ch {
            b'\'' | b'"' | b'`' => {
                i = skip_string(s, i + 1, ch as char)?;
            }
            b'/' => {
                let next = bytes.get(i + 1).copied();
                match next {
                    Some(b'/') => {
                        // line comment - skip until newline
                        i = s[i..]
                            .find('\n')
                            .map(|pos| i + pos + 1)
                            .unwrap_or(bytes.len());
                    }
                    Some(b'*') => {
                        // block comment - skip until */
                        i = s[i + 2..]
                            .find("*/")
                            .map(|pos| i + 2 + pos + 2)
                            .unwrap_or(bytes.len());
                    }
                    _ => {
                        i += 1;
                    }
                }
            }
            c if c == open as u8 => {
                depth += 1;
                i += 1;
            }
            c if c == close as u8 => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    None
}

/// Find the LAST occurrence of the keyword `kw` at bracket depth 0.
///
/// Same as `find_keyword_at_depth_zero` but returns the last match instead of the first.
/// Used in TS mode for `{#each}` blocks where TypeScript `as` assertions
/// may precede the Svelte `as` keyword.
pub fn find_last_keyword_at_depth_zero(s: &str, kw: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth: u32 = 0;
    let mut i = 0;
    let mut last_match: Option<usize> = None;

    while i < bytes.len() {
        if depth == 0 && bytes[i..].starts_with(kw.as_bytes()) {
            last_match = Some(i);
            i += kw.len();
            continue;
        }

        let ch = bytes[i];
        match ch {
            b'\'' | b'"' | b'`' => {
                i = skip_string(s, i + 1, ch as char).unwrap_or(bytes.len());
            }
            b'{' | b'(' | b'[' => {
                depth += 1;
                i += 1;
            }
            b'}' | b')' | b']' => {
                depth = depth.saturating_sub(1);
                i += 1;
            }
            b'/' => {
                let next = bytes.get(i + 1).copied();
                match next {
                    Some(b'/') => {
                        i = s[i..]
                            .find('\n')
                            .map(|pos| i + pos + 1)
                            .unwrap_or(bytes.len());
                    }
                    Some(b'*') => {
                        i = s[i + 2..]
                            .find("*/")
                            .map(|pos| i + 2 + pos + 2)
                            .unwrap_or(bytes.len());
                    }
                    _ => {
                        i += 1;
                    }
                }
            }
            _ => {
                i += 1;
            }
        }
    }

    last_match
}

/// Find the end position of the keyword `kw` at bracket depth 0.
///
/// `s` starts after the opening bracket/whitespace.
/// Returns the byte offset where the keyword starts, or `None` if not found.
///
/// Handles nested brackets {}/()/[] and string/template literals.
pub fn find_keyword_at_depth_zero(s: &str, kw: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth: u32 = 0;
    let mut i = 0;

    while i < bytes.len() {
        if depth == 0 && bytes[i..].starts_with(kw.as_bytes()) {
            return Some(i);
        }

        let ch = bytes[i];
        match ch {
            b'\'' | b'"' | b'`' => {
                i = skip_string(s, i + 1, ch as char).unwrap_or(bytes.len());
            }
            b'{' | b'(' | b'[' => {
                depth += 1;
                i += 1;
            }
            b'}' | b')' | b']' => {
                depth = depth.saturating_sub(1);
                i += 1;
            }
            b'/' => {
                let next = bytes.get(i + 1).copied();
                match next {
                    Some(b'/') => {
                        i = s[i..]
                            .find('\n')
                            .map(|pos| i + pos + 1)
                            .unwrap_or(bytes.len());
                    }
                    Some(b'*') => {
                        i = s[i + 2..]
                            .find("*/")
                            .map(|pos| i + 2 + pos + 2)
                            .unwrap_or(bytes.len());
                    }
                    _ => {
                        i += 1;
                    }
                }
            }
            _ => {
                i += 1;
            }
        }
    }

    None
}

/// Find the byte offset of the first character in `chars` at bracket depth 0.
///
/// Handles nested brackets {}/()/[] and string/template literals.
pub fn find_char_at_depth_zero(s: &str, chars: &[char]) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth: u32 = 0;
    let mut i = 0;

    while i < bytes.len() {
        let ch = bytes[i];

        if depth == 0 {
            // Check if current char is a terminator (for ASCII chars only)
            let c = ch as char;
            if chars.contains(&c) && ch < 128 {
                return Some(i);
            }
        }

        match ch {
            b'\'' | b'"' | b'`' => {
                i = skip_string(s, i + 1, ch as char).unwrap_or(bytes.len());
            }
            b'{' | b'(' | b'[' => {
                depth += 1;
                i += 1;
            }
            b'}' | b')' | b']' => {
                depth = depth.saturating_sub(1);
                i += 1;
            }
            b'/' => {
                let next = bytes.get(i + 1).copied();
                match next {
                    Some(b'/') => {
                        i = s[i..]
                            .find('\n')
                            .map(|pos| i + pos + 1)
                            .unwrap_or(bytes.len());
                    }
                    Some(b'*') => {
                        i = s[i + 2..]
                            .find("*/")
                            .map(|pos| i + 2 + pos + 2)
                            .unwrap_or(bytes.len());
                    }
                    _ => {
                        i += 1;
                    }
                }
            }
            _ => {
                i += 1;
            }
        }
    }

    None
}

/// Skip past a string literal (starting after the opening quote).
/// Returns the byte offset just past the closing quote.
fn skip_string(s: &str, start: usize, quote: char) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut i = start;

    while i < bytes.len() {
        let ch = bytes[i];
        if ch == b'\\' {
            i += 2; // skip escape
            continue;
        }
        if ch == quote as u8 {
            return Some(i + 1); // past closing quote
        }
        // Template literal: handle ${ }
        if quote == '`' && ch == b'$' && bytes.get(i + 1) == Some(&b'{') {
            i += 2; // skip ${
            // Find matching }
            if let Some(end) = find_matching_bracket(&s[i..], '{') {
                i += end + 1; // past the }
            } else {
                return None;
            }
            continue;
        }
        // Non-template strings can't span lines
        if quote != '`' && ch == b'\n' {
            return Some(i); // treat newline as end for non-template strings
        }
        i += 1;
    }

    None
}

// --- Winnow parser wrappers ---

/// Consume `{` ... `}` and return the inner content as `&str` (zero-allocation).
pub fn scan_expression_content<'i>(input: &mut ParserInput<'i>) -> ParseResult<&'i str> {
    // Consume opening {
    let _: &str = take(1usize).parse_next(input)?;

    let remaining: &str = &input.input;
    let end = find_matching_bracket(remaining, '{').ok_or(winnow::error::ContextError::new())?;

    // Take the content (everything before the closing })
    let content: &str = take(end).parse_next(input)?;

    // Consume closing }
    let _: &str = take(1usize).parse_next(input)?;

    Ok(content)
}

/// Read content until `}` at bracket depth 0, without consuming the `}`.
/// Returns the content as `&str` (zero-allocation).
pub fn read_until_close_brace<'i>(input: &mut ParserInput<'i>) -> ParseResult<&'i str> {
    let remaining: &str = &input.input;
    let end = find_matching_bracket_for_close_char(remaining, '}')
        .ok_or(winnow::error::ContextError::new())?;

    take(end).parse_next(input)
}

/// Read content until a keyword at bracket depth 0, without consuming the keyword.
/// Returns the content as `&str` (zero-allocation).
pub fn read_until_keyword_balanced<'a, 'i>(
    input: &mut ParserInput<'i>,
    keyword: &'a str,
) -> ParseResult<&'i str> {
    let remaining: &str = &input.input;
    let end =
        find_keyword_at_depth_zero(remaining, keyword).ok_or(winnow::error::ContextError::new())?;

    take(end).parse_next(input)
}

/// Read content until one of the given chars at bracket depth 0, without consuming.
/// Returns the content as `&str` (zero-allocation).
pub fn read_until_chars_balanced<'i>(
    input: &mut ParserInput<'i>,
    chars: &[char],
) -> ParseResult<&'i str> {
    let remaining: &str = &input.input;
    let end = find_char_at_depth_zero(remaining, chars).ok_or(winnow::error::ContextError::new())?;

    take(end).parse_next(input)
}

/// Find position of `close` at depth 0, without requiring a matching open bracket.
/// Used when we're already inside the brackets and just need to find the close.
pub fn find_matching_bracket_for_close_char(s: &str, close: char) -> Option<usize> {
    let open = match close {
        '}' => '{',
        ')' => '(',
        ']' => '[',
        _ => return None,
    };

    let bytes = s.as_bytes();
    let mut depth: u32 = 0;
    let mut i = 0;

    while i < bytes.len() {
        let ch = bytes[i];
        match ch {
            b'\'' | b'"' | b'`' => {
                i = skip_string(s, i + 1, ch as char).unwrap_or(bytes.len());
            }
            b'/' => {
                let next = bytes.get(i + 1).copied();
                match next {
                    Some(b'/') => {
                        i = s[i..]
                            .find('\n')
                            .map(|pos| i + pos + 1)
                            .unwrap_or(bytes.len());
                    }
                    Some(b'*') => {
                        i = s[i + 2..]
                            .find("*/")
                            .map(|pos| i + 2 + pos + 2)
                            .unwrap_or(bytes.len());
                    }
                    _ => {
                        i += 1;
                    }
                }
            }
            c if c == open as u8 => {
                depth += 1;
                i += 1;
            }
            c if c == close as u8 => {
                if depth == 0 {
                    return Some(i);
                }
                depth -= 1;
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    None
}
