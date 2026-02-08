//! Shared patterns for validation and parsing.
//!
//! Reference: `compiler/phases/patterns.js`
//!
//! Most patterns are implemented as inline functions for performance,
//! avoiding regex overhead where possible.

pub fn is_valid_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

pub fn is_heading_tag(name: &str) -> bool {
    matches!(name, "h1" | "h2" | "h3" | "h4" | "h5" | "h6")
}

pub fn starts_with_newline(s: &str) -> bool {
    s.starts_with('\n') || s.starts_with("\r\n")
}

pub fn is_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r' | '\u{000C}')
}

/// Check if a string starts with `javascript:` (case-insensitive, ignoring leading whitespace).
pub fn is_javascript_protocol(s: &str) -> bool {
    let trimmed = s.trim_start();
    trimmed
        .get(..11)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("javascript:"))
}

/// Check if attribute character is illegal in HTML.
pub fn is_illegal_attribute_char(c: char) -> bool {
    matches!(c,
        ' ' | '\t' | '\n' | '\r' | '"' | '\'' | '>' | '/' | '=' | '{' |
        '\x00'..='\x1f' | '\x7f'..='\u{009f}'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_identifier() {
        assert!(is_valid_identifier("foo"));
        assert!(is_valid_identifier("_bar"));
        assert!(is_valid_identifier("$baz"));
        assert!(is_valid_identifier("a1b2"));
        assert!(!is_valid_identifier("1foo"));
        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("foo-bar"));
        assert!(!is_valid_identifier("foo bar"));
    }

    #[test]
    fn test_heading_tag() {
        assert!(is_heading_tag("h1"));
        assert!(is_heading_tag("h6"));
        assert!(!is_heading_tag("h0"));
        assert!(!is_heading_tag("h7"));
        assert!(!is_heading_tag("div"));
    }

    #[test]
    fn test_javascript_protocol() {
        assert!(is_javascript_protocol("javascript:alert(1)"));
        assert!(is_javascript_protocol("JavaScript:alert(1)"));
        assert!(is_javascript_protocol("  javascript:void(0)"));
        assert!(!is_javascript_protocol("https://example.com"));
        assert!(!is_javascript_protocol("java"));
    }

    #[test]
    fn test_illegal_attribute_char() {
        assert!(is_illegal_attribute_char('"'));
        assert!(is_illegal_attribute_char('\''));
        assert!(is_illegal_attribute_char('>'));
        assert!(is_illegal_attribute_char('{'));
        assert!(is_illegal_attribute_char('\0'));
        assert!(!is_illegal_attribute_char('a'));
        assert!(!is_illegal_attribute_char('-'));
    }
}
