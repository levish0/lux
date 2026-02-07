/// JavaScript reserved words.
///
/// Reference: `utils.js` lines 43-100
use phf::phf_set;

pub static RESERVED_WORDS: phf::Set<&str> = phf_set! {
    "arguments", "await", "break", "case", "catch", "class", "const",
    "continue", "debugger", "default", "delete", "do", "else", "enum",
    "eval", "export", "extends", "false", "finally", "for", "function",
    "if", "implements", "import", "in", "instanceof", "interface", "let",
    "new", "null", "package", "private", "protected", "public", "return",
    "static", "super", "switch", "this", "throw", "true", "try", "typeof",
    "var", "void", "while", "with", "yield",
};

pub fn is_reserved(word: &str) -> bool {
    RESERVED_WORDS.contains(word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reserved_words() {
        assert!(is_reserved("class"));
        assert!(is_reserved("return"));
        assert!(is_reserved("yield"));
        assert!(is_reserved("await"));
        assert!(!is_reserved("foo"));
        assert!(!is_reserved("bar"));
    }
}
