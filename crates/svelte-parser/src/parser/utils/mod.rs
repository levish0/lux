//! Utility functions.
const VOID_ELEMENT_NAMES: &[&str] = &[
    "area", "base", "br", "col", "command", "embed", "hr", "img", "input", "keygen", "link",
    "meta", "param", "source", "track", "wbr",
];

/// Returns `true` if `name` is of a void element.
pub fn is_void(name: &str) -> bool {
    VOID_ELEMENT_NAMES.contains(&name) || name.eq_ignore_ascii_case("!doctype")
}

const RESERVED_WORDS: &[&str] = &[
    "arguments",
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "eval",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "implements",
    "import",
    "in",
    "instanceof",
    "interface",
    "let",
    "new",
    "null",
    "package",
    "private",
    "protected",
    "public",
    "return",
    "static",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "yield",
];

/// Returns `true` if `word` is a reserved JavaScript keyword.
pub fn is_reserved(word: &str) -> bool {
    RESERVED_WORDS.contains(&word)
}

// ─── HTML Tree Validation ────────────────────────────────────────
// Port of reference `src/html-tree-validation.js`

/// Autoclosing rules: `direct` means only immediate children trigger auto-close,
/// `descendant` means any descendant triggers auto-close.
enum AutoCloseRule {
    Direct(&'static [&'static str]),
    Descendant(&'static [&'static str]),
}

fn autoclosing_rule(tag: &str) -> Option<AutoCloseRule> {
    match tag {
        "li" => Some(AutoCloseRule::Direct(&["li"])),
        "dt" => Some(AutoCloseRule::Descendant(&["dt", "dd"])),
        "dd" => Some(AutoCloseRule::Descendant(&["dt", "dd"])),
        "p" => Some(AutoCloseRule::Descendant(&[
            "address",
            "article",
            "aside",
            "blockquote",
            "div",
            "dl",
            "fieldset",
            "footer",
            "form",
            "h1",
            "h2",
            "h3",
            "h4",
            "h5",
            "h6",
            "header",
            "hgroup",
            "hr",
            "main",
            "menu",
            "nav",
            "ol",
            "p",
            "pre",
            "section",
            "table",
            "ul",
        ])),
        "rt" => Some(AutoCloseRule::Descendant(&["rt", "rp"])),
        "rp" => Some(AutoCloseRule::Descendant(&["rt", "rp"])),
        "optgroup" => Some(AutoCloseRule::Descendant(&["optgroup"])),
        "option" => Some(AutoCloseRule::Descendant(&["option", "optgroup"])),
        "thead" => Some(AutoCloseRule::Direct(&["tbody", "tfoot"])),
        "tbody" => Some(AutoCloseRule::Direct(&["tbody", "tfoot"])),
        "tfoot" => Some(AutoCloseRule::Direct(&["tbody"])),
        "tr" => Some(AutoCloseRule::Direct(&["tr", "tbody"])),
        "td" => Some(AutoCloseRule::Direct(&["td", "th", "tr"])),
        "th" => Some(AutoCloseRule::Direct(&["td", "th", "tr"])),
        _ => None,
    }
}

/// Returns true if the `current` tag should be auto-closed when `next` tag is encountered.
/// Port of reference `closing_tag_omitted` from html-tree-validation.js.
pub fn closing_tag_omitted(current: &str, next: &str) -> bool {
    if let Some(rule) = autoclosing_rule(current) {
        let list = match rule {
            AutoCloseRule::Direct(tags) | AutoCloseRule::Descendant(tags) => tags,
        };
        return list.contains(&next);
    }
    false
}
