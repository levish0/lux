//! Reserved words utilities.

use phf::phf_set;

static RESERVED_WORDS: phf::Set<&'static str> = phf_set! {
    "arguments", "await", "break", "case", "catch", "class", "const", "continue",
    "debugger", "default", "delete", "do", "else", "enum", "eval", "export",
    "extends", "false", "finally", "for", "function", "if", "implements",
    "import", "in", "instanceof", "interface", "let", "new", "null", "package",
    "private", "protected", "public", "return", "static", "super", "switch",
    "this", "throw", "true", "try", "typeof", "var", "void", "while", "with", "yield"
};

/// Returns `true` if `word` is a reserved JavaScript keyword.
pub fn is_reserved(word: &str) -> bool {
    RESERVED_WORDS.contains(word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_reserved() {
        assert!(is_reserved("await"));
        assert!(is_reserved("class"));
        assert!(is_reserved("function"));
        assert!(is_reserved("return"));
        assert!(!is_reserved("foo"));
        assert!(!is_reserved("myVariable"));
    }
}
