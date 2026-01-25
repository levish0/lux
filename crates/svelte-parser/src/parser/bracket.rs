/// Bracket matching utilities.
/// Direct port of reference utils/bracket.js.

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
            let default_brackets = [('{', '}'), ('(', ')'), ('[', ']')];
            // Find matching } using match_bracket logic
            let end = match_bracket(template, i - 1, &default_brackets)?;
            i = end;
        }
    }

    None // unterminated string
}