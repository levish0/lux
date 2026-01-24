use regex::Regex;
use std::sync::LazyLock;

pub static REGEX_WHITESPACE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s").unwrap());

pub static REGEX_WHITESPACES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());

pub static REGEX_STARTS_WITH_NEWLINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\r?\n").unwrap());

pub static REGEX_STARTS_WITH_WHITESPACE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s").unwrap());

pub static REGEX_STARTS_WITH_WHITESPACES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[ \t\r\n]+").unwrap());

pub static REGEX_ENDS_WITH_WHITESPACE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s$").unwrap());

pub static REGEX_ENDS_WITH_WHITESPACES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[ \t\r\n]+$").unwrap());

/// Not `\S` because that also removes explicit whitespace defined through things like `&nbsp;`
pub static REGEX_NOT_WHITESPACE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[^ \t\r\n]").unwrap());

/// Not `\s+` because that also includes explicit whitespace defined through things like `&nbsp;`
pub static REGEX_WHITESPACES_STRICT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[ \t\n\r\f]+").unwrap());

pub static REGEX_ONLY_WHITESPACES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[ \t\n\r\f]+$").unwrap());

pub static REGEX_NOT_NEWLINE_CHARACTERS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[^\n]").unwrap());

pub static REGEX_IS_VALID_IDENTIFIER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z_$][a-zA-Z_$0-9]*$").unwrap());

/// Used in replace all to remove all invalid chars from a literal identifier.
pub static REGEX_INVALID_IDENTIFIER_CHARS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(^[^a-zA-Z_$]|[^a-zA-Z0-9_$])").unwrap());

pub static REGEX_STARTS_WITH_VOWEL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[aeiou]").unwrap());

pub static REGEX_HEADING_TAGS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^h[1-6]$").unwrap());

pub static REGEX_ILLEGAL_ATTRIBUTE_CHARACTER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(^[0-9\-.])|[\^$@%&#?!|()\[\]{}*+~;]"#).unwrap());

pub static REGEX_BIDIRECTIONAL_CONTROL_CHARACTERS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[\u{202a}\u{202b}\u{202c}\u{202d}\u{202e}\u{2066}\u{2067}\u{2068}\u{2069}]+")
        .unwrap()
});

pub static REGEX_JS_PREFIX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^\W*javascript:").unwrap());

pub static REGEX_REDUNDANT_IMG_ALT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\b(image|picture|photo)\b").unwrap());

/// Used in tag
pub static REGEX_WHITESPACE_WITH_CLOSING_CURLY_BRACE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*}").unwrap());
