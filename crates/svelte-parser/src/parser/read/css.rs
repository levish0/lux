use svelte_ast::css::*;
use svelte_ast::span::Span;

use crate::error::ErrorKind;
use crate::parser::{ParseError, Parser};

/// Port of reference `read/style.js` → `read_body`.
/// Reads CSS rules until `finished` returns true.
pub fn read_body(
    parser: &mut Parser<'_>,
    finished: impl Fn(&Parser<'_>) -> bool,
) -> Result<Vec<StyleSheetChild>, ParseError> {
    let mut children = Vec::new();

    loop {
        allow_comment_or_whitespace(parser);
        if finished(parser) {
            break;
        }
        if parser.match_str("@") {
            children.push(StyleSheetChild::Atrule(read_at_rule(parser)?));
        } else {
            children.push(StyleSheetChild::Rule(read_rule(parser)?));
        }
    }

    Ok(children)
}

/// Port of reference `read_at_rule`.
fn read_at_rule(parser: &mut Parser<'_>) -> Result<CssAtrule, ParseError> {
    let start = parser.index;
    parser.eat_required("@")?;

    let name = read_identifier(parser)?;
    let prelude = read_value(parser)?;

    let block = if parser.match_str("{") {
        Some(read_block(parser)?)
    } else {
        parser.eat_required(";")?;
        None
    };

    Ok(CssAtrule {
        span: Span::new(start, parser.index),
        name,
        prelude,
        block,
    })
}

/// Port of reference `read_rule`.
fn read_rule(parser: &mut Parser<'_>) -> Result<CssRule, ParseError> {
    let start = parser.index;
    let prelude = read_selector_list(parser, false)?;
    let block = read_block(parser)?;

    Ok(CssRule {
        span: Span::new(start, parser.index),
        prelude,
        block,
    })
}

/// Port of reference `read_selector_list`.
fn read_selector_list(
    parser: &mut Parser<'_>,
    inside_pseudo_class: bool,
) -> Result<SelectorList, ParseError> {
    let mut children = Vec::new();

    allow_comment_or_whitespace(parser);
    let start = parser.index;

    while parser.index < parser.template.len() {
        children.push(read_selector(parser, inside_pseudo_class)?);
        let end = parser.index;

        allow_comment_or_whitespace(parser);

        if inside_pseudo_class && parser.match_str(")") {
            return Ok(SelectorList {
                span: Span::new(start, end),
                children,
            });
        }
        if !inside_pseudo_class && parser.match_str("{") {
            return Ok(SelectorList {
                span: Span::new(start, end),
                children,
            });
        }

        parser.eat_required(",")?;
        allow_comment_or_whitespace(parser);
    }

    Err(parser.error(
        ErrorKind::UnexpectedEof,
        parser.template.len(),
        "Unexpected end of input".to_string(),
    ))
}

/// Port of reference `read_selector`.
fn read_selector(
    parser: &mut Parser<'_>,
    inside_pseudo_class: bool,
) -> Result<ComplexSelector, ParseError> {
    let list_start = parser.index;
    let mut children: Vec<RelativeSelector> = Vec::new();

    let mut relative_selector = RelativeSelector {
        span: Span::new(parser.index, 0),
        combinator: None,
        selectors: Vec::new(),
    };

    while parser.index < parser.template.len() {
        let start = parser.index;

        if parser.eat("&") {
            relative_selector
                .selectors
                .push(SimpleSelector::NestingSelector(NestingSelector {
                    span: Span::new(start, parser.index),
                    name: "&".to_string(),
                }));
        } else if parser.eat("*") {
            let mut name = "*".to_string();
            if parser.eat("|") {
                name = read_identifier(parser)?;
            }
            relative_selector
                .selectors
                .push(SimpleSelector::TypeSelector(TypeSelector {
                    span: Span::new(start, parser.index),
                    name,
                }));
        } else if parser.eat("#") {
            let name = read_identifier(parser)?;
            relative_selector
                .selectors
                .push(SimpleSelector::IdSelector(IdSelector {
                    span: Span::new(start, parser.index),
                    name,
                }));
        } else if parser.eat(".") {
            let name = read_identifier(parser)?;
            relative_selector
                .selectors
                .push(SimpleSelector::ClassSelector(ClassSelector {
                    span: Span::new(start, parser.index),
                    name,
                }));
        } else if parser.eat("::") {
            let name = read_identifier(parser)?;
            relative_selector
                .selectors
                .push(SimpleSelector::PseudoElementSelector(
                    PseudoElementSelector {
                        span: Span::new(start, parser.index),
                        name,
                    },
                ));
            // Read inner selectors of pseudo element to ensure it parses correctly
            if parser.eat("(") {
                read_selector_list(parser, true)?;
                parser.eat_required(")")?;
            }
        } else if parser.eat(":") {
            let name = read_identifier(parser)?;
            let args = if parser.eat("(") {
                let list = read_selector_list(parser, true)?;
                parser.eat_required(")")?;
                Some(Box::new(list))
            } else {
                None
            };
            relative_selector
                .selectors
                .push(SimpleSelector::PseudoClassSelector(PseudoClassSelector {
                    span: Span::new(start, parser.index),
                    name,
                    args,
                }));
        } else if parser.eat("[") {
            parser.allow_whitespace();
            let name = read_identifier(parser)?;
            parser.allow_whitespace();

            let matcher = read_matcher(parser);
            let value = if matcher.is_some() {
                parser.allow_whitespace();
                Some(read_attribute_value(parser)?)
            } else {
                None
            };

            parser.allow_whitespace();
            let flags = read_attribute_flags(parser);
            parser.allow_whitespace();
            parser.eat_required("]")?;

            relative_selector
                .selectors
                .push(SimpleSelector::AttributeSelector(AttributeSelector {
                    span: Span::new(start, parser.index),
                    name,
                    matcher,
                    value,
                    flags,
                }));
        } else if inside_pseudo_class && match_nth_of(parser) {
            // nth of matcher must come before combinator matcher
            let value = read_nth_of(parser);
            relative_selector.selectors.push(SimpleSelector::Nth(Nth {
                span: Span::new(start, parser.index),
                value,
            }));
        } else if match_percentage(parser) {
            let value = read_percentage(parser);
            relative_selector
                .selectors
                .push(SimpleSelector::Percentage(Percentage {
                    span: Span::new(start, parser.index),
                    value,
                }));
        } else if !match_combinator(parser) {
            let mut name = read_identifier(parser)?;
            if parser.eat("|") {
                name = read_identifier(parser)?;
            }
            relative_selector
                .selectors
                .push(SimpleSelector::TypeSelector(TypeSelector {
                    span: Span::new(start, parser.index),
                    name,
                }));
        }

        let index = parser.index;
        allow_comment_or_whitespace(parser);

        if parser.match_str(",")
            || (if inside_pseudo_class {
                parser.match_str(")")
            } else {
                parser.match_str("{")
            })
        {
            // Rewind
            parser.index = index;
            relative_selector.span = Span::new(relative_selector.span.start, index);
            children.push(relative_selector);

            return Ok(ComplexSelector {
                span: Span::new(list_start, index),
                children,
            });
        }

        parser.index = index;
        let combinator = read_combinator(parser);

        if let Some(comb) = combinator {
            if !relative_selector.selectors.is_empty() {
                relative_selector.span = Span::new(relative_selector.span.start, index);
                children.push(relative_selector);
            }

            let comb_start = comb.span.start;
            relative_selector = RelativeSelector {
                span: Span::new(comb_start, 0),
                combinator: Some(comb),
                selectors: Vec::new(),
            };

            parser.allow_whitespace();

            if parser.match_str(",")
                || (if inside_pseudo_class {
                    parser.match_str(")")
                } else {
                    parser.match_str("{")
                })
            {
                return Err(parser.error(
                    ErrorKind::CssSelectorInvalid,
                    parser.index,
                    "Invalid selector".to_string(),
                ));
            }
        }
    }

    Err(parser.error(
        ErrorKind::UnexpectedEof,
        parser.template.len(),
        "Unexpected end of input".to_string(),
    ))
}

/// Port of reference `read_combinator`.
fn read_combinator(parser: &mut Parser<'_>) -> Option<CssCombinator> {
    let start = parser.index;
    parser.allow_whitespace();

    let index = parser.index;

    // Try to match combinator: +, ~, >, ||
    let bytes = parser.template.as_bytes();
    let name = if index < bytes.len() {
        match bytes[index] {
            b'+' | b'~' | b'>' => {
                parser.index += 1;
                Some(&parser.template[index..parser.index])
            }
            b'|' if index + 1 < bytes.len() && bytes[index + 1] == b'|' => {
                parser.index += 2;
                Some(&parser.template[index..parser.index])
            }
            _ => None,
        }
    } else {
        None
    };

    if let Some(name) = name {
        let end = parser.index;
        parser.allow_whitespace();
        return Some(CssCombinator {
            span: Span::new(index, end),
            name: name.to_string(),
        });
    }

    // If whitespace was consumed, it's a descendant combinator
    if parser.index != start {
        return Some(CssCombinator {
            span: Span::new(start, parser.index),
            name: " ".to_string(),
        });
    }

    None
}

/// Port of reference `read_block`.
fn read_block(parser: &mut Parser<'_>) -> Result<CssBlock, ParseError> {
    let start = parser.index;
    parser.eat_required("{")?;

    let mut children = Vec::new();

    while parser.index < parser.template.len() {
        allow_comment_or_whitespace(parser);
        if parser.match_str("}") {
            break;
        }
        children.push(read_block_item(parser)?);
    }

    parser.eat_required("}")?;

    Ok(CssBlock {
        span: Span::new(start, parser.index),
        children,
    })
}

/// Port of reference `read_block_item`.
fn read_block_item(parser: &mut Parser<'_>) -> Result<CssBlockChild, ParseError> {
    if parser.match_str("@") {
        return Ok(CssBlockChild::Atrule(read_at_rule(parser)?));
    }

    // Read ahead to determine if declaration or nested rule
    let start = parser.index;
    read_value(parser)?;
    let ch = parser.template.as_bytes().get(parser.index).copied();
    parser.index = start;

    if ch == Some(b'{') {
        Ok(CssBlockChild::Rule(read_rule(parser)?))
    } else {
        Ok(CssBlockChild::Declaration(read_declaration(parser)?))
    }
}

/// Port of reference `read_declaration`.
fn read_declaration(parser: &mut Parser<'_>) -> Result<CssDeclaration, ParseError> {
    let start = parser.index;

    // Read property: until whitespace or colon
    let property = read_until_whitespace_or_colon(parser);
    parser.allow_whitespace();
    parser.eat_required(":")?;
    let _index = parser.index;
    parser.allow_whitespace();

    let value = read_value(parser)?;

    if value.is_empty() && !property.starts_with("--") {
        return Err(parser.error(
            ErrorKind::CssEmptyDeclaration,
            start,
            "Declaration value is empty".to_string(),
        ));
    }

    let end = parser.index;

    if !parser.match_str("}") {
        parser.eat_required(";")?;
    }

    Ok(CssDeclaration {
        span: Span::new(start, end),
        property,
        value,
    })
}

/// Port of reference `read_value`.
fn read_value(parser: &mut Parser<'_>) -> Result<String, ParseError> {
    let mut value = String::new();
    let mut escaped = false;
    let mut in_url = false;
    let mut quote_mark: Option<char> = None;

    while parser.index < parser.template.len() {
        // Get the current character (handling multi-byte UTF-8)
        let remaining = &parser.template[parser.index..];
        let ch = match remaining.chars().next() {
            Some(c) => c,
            None => break,
        };
        let ch_len = ch.len_utf8();

        if escaped {
            value.push('\\');
            value.push(ch);
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if Some(ch) == quote_mark {
            quote_mark = None;
        } else if ch == ')' {
            in_url = false;
        } else if quote_mark.is_none() && (ch == '"' || ch == '\'') {
            quote_mark = Some(ch);
        } else if ch == '(' && value.ends_with("url") {
            in_url = true;
        } else if (ch == ';' || ch == '{' || ch == '}') && !in_url && quote_mark.is_none() {
            return Ok(value.trim().to_string());
        }

        value.push(ch);
        parser.index += ch_len;
    }

    Err(parser.error(
        ErrorKind::UnexpectedEof,
        parser.template.len(),
        "Unexpected end of input".to_string(),
    ))
}

/// Port of reference `read_attribute_value` (CSS attribute selector values).
fn read_attribute_value(parser: &mut Parser<'_>) -> Result<String, ParseError> {
    let mut value = String::new();
    let mut escaped = false;

    let quote_mark = if parser.eat("\"") {
        Some('"')
    } else if parser.eat("'") {
        Some('\'')
    } else {
        None
    };

    while parser.index < parser.template.len() {
        // Get the current character (handling multi-byte UTF-8)
        let remaining = &parser.template[parser.index..];
        let ch = match remaining.chars().next() {
            Some(c) => c,
            None => break,
        };
        let ch_len = ch.len_utf8();

        if escaped {
            value.push('\\');
            value.push(ch);
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if let Some(q) = quote_mark {
            if ch == q {
                parser.index += ch_len; // consume closing quote
                return Ok(value.trim().to_string());
            } else {
                value.push(ch);
            }
        } else if ch.is_ascii_whitespace() || ch == ']' {
            return Ok(value.trim().to_string());
        } else {
            value.push(ch);
        }

        parser.index += ch_len;
    }

    Err(parser.error(
        ErrorKind::UnexpectedEof,
        parser.template.len(),
        "Unexpected end of input".to_string(),
    ))
}

/// Port of reference `read_identifier` (CSS identifiers).
/// Handles unicode escape sequences and non-ASCII characters.
fn read_identifier(parser: &mut Parser<'_>) -> Result<String, ParseError> {
    let start = parser.index;
    let mut identifier = String::new();

    // Check for leading hyphen-or-digit which is invalid
    if match_leading_hyphen_or_digit(parser) {
        return Err(parser.error(
            ErrorKind::CssExpectedIdentifier,
            start,
            "Expected a valid CSS identifier".to_string(),
        ));
    }

    let bytes = parser.template.as_bytes();

    while parser.index < bytes.len() {
        let ch = bytes[parser.index];

        if ch == b'\\' {
            // Try unicode escape sequence: \[0-9a-fA-F]{1,6}(\r\n|\s)?
            if let Some((codepoint_str, seq_len)) = match_unicode_sequence(parser) {
                if let Ok(cp) = u32::from_str_radix(&codepoint_str, 16) {
                    if let Some(c) = char::from_u32(cp) {
                        identifier.push(c);
                    }
                }
                parser.index += seq_len;
            } else {
                // Regular escape: just include backslash + next char
                identifier.push('\\');
                if parser.index + 1 < bytes.len() {
                    identifier.push(bytes[parser.index + 1] as char);
                    parser.index += 2;
                } else {
                    parser.index += 1;
                }
            }
        } else if ch >= 160 || is_valid_identifier_char(ch) {
            // Non-ASCII (>= 160) or valid ASCII identifier char
            if ch >= 128 {
                // Handle multi-byte UTF-8
                let remaining = &parser.template[parser.index..];
                let mut chars = remaining.chars();
                if let Some(c) = chars.next() {
                    if c as u32 >= 160 || is_valid_identifier_char(ch) {
                        identifier.push(c);
                        parser.index += c.len_utf8();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                identifier.push(ch as char);
                parser.index += 1;
            }
        } else {
            break;
        }
    }

    if identifier.is_empty() {
        return Err(parser.error(
            ErrorKind::CssExpectedIdentifier,
            start,
            "Expected a valid CSS identifier".to_string(),
        ));
    }

    Ok(identifier)
}

/// Port of reference `allow_comment_or_whitespace`.
fn allow_comment_or_whitespace(parser: &mut Parser<'_>) {
    parser.allow_whitespace();
    while parser.match_str("/*") || parser.match_str("<!--") {
        if parser.eat("/*") {
            parser.read_until_str("*/");
            let _ = parser.eat("*/");
        }
        if parser.eat("<!--") {
            parser.read_until_str("-->");
            let _ = parser.eat("-->");
        }
        parser.allow_whitespace();
    }
}

// ─── Helper Functions ─────────────────────────────────────────

/// Read until whitespace or colon (for declaration property).
fn read_until_whitespace_or_colon(parser: &mut Parser<'_>) -> String {
    let start = parser.index;
    let bytes = parser.template.as_bytes();
    while parser.index < bytes.len() {
        let ch = bytes[parser.index];
        if ch.is_ascii_whitespace() || ch == b':' {
            break;
        }
        parser.index += 1;
    }
    parser.template[start..parser.index].to_string()
}

/// Check if current position starts with a leading hyphen or digit (invalid for CSS identifier start).
fn match_leading_hyphen_or_digit(parser: &Parser<'_>) -> bool {
    let bytes = parser.template.as_bytes();
    if parser.index >= bytes.len() {
        return false;
    }
    let ch = bytes[parser.index];
    if ch.is_ascii_digit() {
        return true;
    }
    if ch == b'-' && parser.index + 1 < bytes.len() && bytes[parser.index + 1].is_ascii_digit() {
        return true;
    }
    false
}

/// Check if byte is a valid CSS identifier character [a-zA-Z0-9_-].
fn is_valid_identifier_char(ch: u8) -> bool {
    ch.is_ascii_alphanumeric() || ch == b'_' || ch == b'-'
}

/// Try to match a unicode escape sequence at current position.
/// Returns (hex_string, total_sequence_length) if matched.
fn match_unicode_sequence(parser: &Parser<'_>) -> Option<(String, usize)> {
    let bytes = parser.template.as_bytes();
    let start = parser.index;

    if start >= bytes.len() || bytes[start] != b'\\' {
        return None;
    }

    let mut i = start + 1;
    if i >= bytes.len() || !bytes[i].is_ascii_hexdigit() {
        return None;
    }

    // Read 1-6 hex digits
    let hex_start = i;
    let mut hex_count = 0;
    while i < bytes.len() && bytes[i].is_ascii_hexdigit() && hex_count < 6 {
        i += 1;
        hex_count += 1;
    }

    let hex_str = parser.template[hex_start..i].to_string();

    // Optional trailing whitespace: \r\n or single whitespace char
    if i + 1 < bytes.len() && bytes[i] == b'\r' && bytes[i + 1] == b'\n' {
        i += 2;
    } else if i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }

    Some((hex_str, i - start))
}

/// Read REGEX_MATCHER: optional [~^$*|] then =
fn read_matcher(parser: &mut Parser<'_>) -> Option<String> {
    let bytes = parser.template.as_bytes();
    let start = parser.index;

    if start >= bytes.len() {
        return None;
    }

    let mut i = start;
    // Optional prefix
    if i < bytes.len() && matches!(bytes[i], b'~' | b'^' | b'$' | b'*' | b'|') {
        i += 1;
    }
    // Require =
    if i < bytes.len() && bytes[i] == b'=' {
        i += 1;
        let result = parser.template[start..i].to_string();
        parser.index = i;
        Some(result)
    } else {
        None
    }
}

/// Read REGEX_ATTRIBUTE_FLAGS: [a-zA-Z]+
fn read_attribute_flags(parser: &mut Parser<'_>) -> Option<String> {
    let bytes = parser.template.as_bytes();
    let start = parser.index;
    while parser.index < bytes.len() && bytes[parser.index].is_ascii_alphabetic() {
        parser.index += 1;
    }
    if parser.index > start {
        Some(parser.template[start..parser.index].to_string())
    } else {
        None
    }
}

/// Check if current position matches a combinator pattern: +, ~, >, ||
fn match_combinator(parser: &Parser<'_>) -> bool {
    let bytes = parser.template.as_bytes();
    if parser.index >= bytes.len() {
        return false;
    }
    match bytes[parser.index] {
        b'+' | b'~' | b'>' => true,
        b'|' => parser.index + 1 < bytes.len() && bytes[parser.index + 1] == b'|',
        _ => false,
    }
}

/// Check if current position matches REGEX_PERCENTAGE: \d+(\.\d+)?%
fn match_percentage(parser: &Parser<'_>) -> bool {
    let bytes = parser.template.as_bytes();
    let mut i = parser.index;
    if i >= bytes.len() || !bytes[i].is_ascii_digit() {
        return false;
    }
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        if i >= bytes.len() || !bytes[i].is_ascii_digit() {
            return false;
        }
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
    }
    i < bytes.len() && bytes[i] == b'%'
}

/// Read a percentage value and advance parser index.
fn read_percentage(parser: &mut Parser<'_>) -> String {
    let start = parser.index;
    let bytes = parser.template.as_bytes();
    while parser.index < bytes.len() && bytes[parser.index].is_ascii_digit() {
        parser.index += 1;
    }
    if parser.index < bytes.len() && bytes[parser.index] == b'.' {
        parser.index += 1;
        while parser.index < bytes.len() && bytes[parser.index].is_ascii_digit() {
            parser.index += 1;
        }
    }
    // Consume the %
    if parser.index < bytes.len() && bytes[parser.index] == b'%' {
        parser.index += 1;
    }
    parser.template[start..parser.index].to_string()
}

/// Check if current position matches REGEX_NTH_OF pattern.
fn match_nth_of(parser: &Parser<'_>) -> bool {
    read_nth_of_len(parser).is_some()
}

/// Read an nth-of value and advance parser index.
fn read_nth_of(parser: &mut Parser<'_>) -> String {
    if let Some(len) = read_nth_of_len(parser) {
        let start = parser.index;
        parser.index += len;
        parser.template[start..parser.index].to_string()
    } else {
        String::new()
    }
}

/// Try to match the REGEX_NTH_OF pattern at current position.
/// Returns the length of the match if successful.
/// Pattern: (even|odd|\+?(\d+|\d*n(\s*[+-]\s*\d+)?)|-\d*n(\s*\+\s*\d+)?)((?=\s*[,)])|\s+of\s+)
fn read_nth_of_len(parser: &Parser<'_>) -> Option<usize> {
    let bytes = parser.template.as_bytes();
    let start = parser.index;
    let len = bytes.len();

    if start >= len {
        return None;
    }

    let mut i = start;

    // Try to match the main part
    if i + 4 <= len && &parser.template[i..i + 4] == "even" {
        i += 4;
    } else if i + 3 <= len && &parser.template[i..i + 3] == "odd" {
        i += 3;
    } else if i < len && bytes[i] == b'-' {
        // -\d*n(\s*\+\s*\d+)?
        i += 1;
        // optional digits
        while i < len && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i >= len || bytes[i] != b'n' {
            return None;
        }
        i += 1;
        // optional \s*\+\s*\d+
        let saved = i;
        let mut j = i;
        while j < len && bytes[j].is_ascii_whitespace() {
            j += 1;
        }
        if j < len && bytes[j] == b'+' {
            j += 1;
            while j < len && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < len && bytes[j].is_ascii_digit() {
                while j < len && bytes[j].is_ascii_digit() {
                    j += 1;
                }
                i = j;
            }
            // If no digit after +, don't consume the + part
        }
        if i == saved {
            // no trailing +\d consumed, that's fine
        }
    } else {
        // \+?(\d+|\d*n(\s*[+-]\s*\d+)?)
        if i < len && bytes[i] == b'+' {
            i += 1;
        }
        // Now need digits or digits followed by n
        let digit_start = i;
        while i < len && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i < len && bytes[i] == b'n' {
            i += 1;
            // optional \s*[+-]\s*\d+
            let saved = i;
            let mut j = i;
            while j < len && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < len && (bytes[j] == b'+' || bytes[j] == b'-') {
                j += 1;
                while j < len && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                if j < len && bytes[j].is_ascii_digit() {
                    while j < len && bytes[j].is_ascii_digit() {
                        j += 1;
                    }
                    i = j;
                }
            }
            if i == saved {
                // no trailing part
            }
        } else if i == digit_start {
            // No digits and no 'n' - not a match
            return None;
        }
        // else: just digits (like "2")
    }

    // Now check the lookahead: (?=\s*[,)]) | \s+of\s+
    let match_end = i;
    let mut j = i;

    // Try \s*[,)]
    while j < len && bytes[j].is_ascii_whitespace() {
        j += 1;
    }
    if j < len && (bytes[j] == b',' || bytes[j] == b')') {
        return Some(match_end - start);
    }

    // Try \s+of\s+
    j = match_end;
    let ws_start = j;
    while j < len && bytes[j].is_ascii_whitespace() {
        j += 1;
    }
    if j > ws_start && j + 2 <= len && &parser.template[j..j + 2] == "of" {
        let after_of = j + 2;
        if after_of < len && bytes[after_of].is_ascii_whitespace() {
            // Include the "of " in the match
            let mut k = after_of;
            while k < len && bytes[k].is_ascii_whitespace() {
                k += 1;
            }
            return Some(k - start);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use crate::parser::Parser;
    use oxc_allocator::Allocator;
    use svelte_ast::css::*;

    #[test]
    fn test_simple_rule() {
        let allocator = Allocator::default();
        let parser = Parser::new("<style>.foo { color: red; }</style>", &allocator, false);
        let root = parser.into_root();
        let css = root.css.unwrap();
        assert_eq!(css.children.len(), 1);
        match &css.children[0] {
            StyleSheetChild::Rule(rule) => {
                assert_eq!(rule.prelude.children.len(), 1);
                let complex = &rule.prelude.children[0];
                assert_eq!(complex.children.len(), 1);
                let rel = &complex.children[0];
                assert_eq!(rel.selectors.len(), 1);
                match &rel.selectors[0] {
                    SimpleSelector::ClassSelector(s) => assert_eq!(s.name, "foo"),
                    _ => panic!("expected ClassSelector"),
                }
                assert_eq!(rule.block.children.len(), 1);
                match &rule.block.children[0] {
                    CssBlockChild::Declaration(d) => {
                        assert_eq!(d.property, "color");
                        assert_eq!(d.value, "red");
                    }
                    _ => panic!("expected Declaration"),
                }
            }
            _ => panic!("expected Rule"),
        }
    }

    #[test]
    fn test_multiple_selectors() {
        let allocator = Allocator::default();
        let parser = Parser::new("<style>.a, .b { margin: 0; }</style>", &allocator, false);
        let root = parser.into_root();
        let css = root.css.unwrap();
        assert_eq!(css.children.len(), 1);
        match &css.children[0] {
            StyleSheetChild::Rule(rule) => {
                assert_eq!(rule.prelude.children.len(), 2);
            }
            _ => panic!("expected Rule"),
        }
    }

    #[test]
    fn test_at_rule() {
        let allocator = Allocator::default();
        let parser = Parser::new(
            "<style>@media (max-width: 600px) { .foo { display: none; } }</style>",
            &allocator,
            false,
        );
        let root = parser.into_root();
        let css = root.css.unwrap();
        assert_eq!(css.children.len(), 1);
        match &css.children[0] {
            StyleSheetChild::Atrule(at) => {
                assert_eq!(at.name, "media");
                assert!(at.block.is_some());
            }
            _ => panic!("expected Atrule"),
        }
    }

    #[test]
    fn test_id_selector() {
        let allocator = Allocator::default();
        let parser = Parser::new("<style>#main { padding: 10px; }</style>", &allocator, false);
        let root = parser.into_root();
        let css = root.css.unwrap();
        match &css.children[0] {
            StyleSheetChild::Rule(rule) => {
                let rel = &rule.prelude.children[0].children[0];
                match &rel.selectors[0] {
                    SimpleSelector::IdSelector(s) => assert_eq!(s.name, "main"),
                    _ => panic!("expected IdSelector"),
                }
            }
            _ => panic!("expected Rule"),
        }
    }

    #[test]
    fn test_descendant_combinator() {
        let allocator = Allocator::default();
        let parser = Parser::new("<style>.a .b { color: blue; }</style>", &allocator, false);
        let root = parser.into_root();
        let css = root.css.unwrap();
        match &css.children[0] {
            StyleSheetChild::Rule(rule) => {
                let complex = &rule.prelude.children[0];
                assert_eq!(complex.children.len(), 2);
                assert!(complex.children[1].combinator.is_some());
                assert_eq!(complex.children[1].combinator.as_ref().unwrap().name, " ");
            }
            _ => panic!("expected Rule"),
        }
    }

    #[test]
    fn test_child_combinator() {
        let allocator = Allocator::default();
        let parser = Parser::new(
            "<style>.a > .b { color: green; }</style>",
            &allocator,
            false,
        );
        let root = parser.into_root();
        let css = root.css.unwrap();
        match &css.children[0] {
            StyleSheetChild::Rule(rule) => {
                let complex = &rule.prelude.children[0];
                assert_eq!(complex.children.len(), 2);
                assert_eq!(complex.children[1].combinator.as_ref().unwrap().name, ">");
            }
            _ => panic!("expected Rule"),
        }
    }

    #[test]
    fn test_pseudo_class() {
        let allocator = Allocator::default();
        let parser = Parser::new("<style>a:hover { color: red; }</style>", &allocator, false);
        let root = parser.into_root();
        let css = root.css.unwrap();
        match &css.children[0] {
            StyleSheetChild::Rule(rule) => {
                let rel = &rule.prelude.children[0].children[0];
                assert_eq!(rel.selectors.len(), 2);
                match &rel.selectors[1] {
                    SimpleSelector::PseudoClassSelector(s) => assert_eq!(s.name, "hover"),
                    _ => panic!("expected PseudoClassSelector"),
                }
            }
            _ => panic!("expected Rule"),
        }
    }

    #[test]
    fn test_attribute_selector() {
        let allocator = Allocator::default();
        let parser = Parser::new(
            "<style>[data-x=\"foo\"] { color: red; }</style>",
            &allocator,
            false,
        );
        let root = parser.into_root();
        let css = root.css.unwrap();
        match &css.children[0] {
            StyleSheetChild::Rule(rule) => {
                let rel = &rule.prelude.children[0].children[0];
                match &rel.selectors[0] {
                    SimpleSelector::AttributeSelector(s) => {
                        assert_eq!(s.name, "data-x");
                        assert_eq!(s.matcher.as_deref(), Some("="));
                        assert_eq!(s.value.as_deref(), Some("foo"));
                    }
                    _ => panic!("expected AttributeSelector"),
                }
            }
            _ => panic!("expected Rule"),
        }
    }

    #[test]
    fn test_content_styles() {
        let allocator = Allocator::default();
        let parser = Parser::new("<style>.foo { color: red; }</style>", &allocator, false);
        let root = parser.into_root();
        let css = root.css.unwrap();
        assert_eq!(css.content.styles, ".foo { color: red; }");
    }

    #[test]
    fn test_complex_css() {
        let template = r#"<style>
    .container {
        display: grid;
        grid-template-columns: 1fr 2fr;
        --custom-var: #333;
    }

    .container.dark {
        background: var(--custom-var);
    }

    @media (max-width: 768px) {
        .container {
            grid-template-columns: 1fr;
        }
    }

    a:hover,
    a:focus {
        text-decoration: underline;
    }
</style>"#;
        let allocator = Allocator::default();
        let parser = Parser::new(template, &allocator, false);
        let root = parser.into_root();
        let css = root.css.unwrap();
        // 4 top-level rules: .container, .container.dark, @media, a:hover/a:focus
        assert_eq!(css.children.len(), 4);
    }

    #[test]
    fn test_pseudo_element() {
        let allocator = Allocator::default();
        let parser = Parser::new(
            "<style>p::before { content: ''; }</style>",
            &allocator,
            false,
        );
        let root = parser.into_root();
        let css = root.css.unwrap();
        match &css.children[0] {
            StyleSheetChild::Rule(rule) => {
                let rel = &rule.prelude.children[0].children[0];
                assert_eq!(rel.selectors.len(), 2);
                match &rel.selectors[1] {
                    SimpleSelector::PseudoElementSelector(s) => assert_eq!(s.name, "before"),
                    _ => panic!("expected PseudoElementSelector"),
                }
            }
            _ => panic!("expected Rule"),
        }
    }

    #[test]
    fn test_nesting_selector() {
        let allocator = Allocator::default();
        let parser = Parser::new(
            "<style>.foo { & .bar { color: red; } }</style>",
            &allocator,
            false,
        );
        let root = parser.into_root();
        let css = root.css.unwrap();
        match &css.children[0] {
            StyleSheetChild::Rule(rule) => {
                assert_eq!(rule.block.children.len(), 1);
                match &rule.block.children[0] {
                    CssBlockChild::Rule(nested) => {
                        let rel = &nested.prelude.children[0].children[0];
                        match &rel.selectors[0] {
                            SimpleSelector::NestingSelector(s) => assert_eq!(s.name, "&"),
                            _ => panic!("expected NestingSelector"),
                        }
                    }
                    _ => panic!("expected nested Rule"),
                }
            }
            _ => panic!("expected Rule"),
        }
    }

    #[test]
    fn test_import_at_rule() {
        let allocator = Allocator::default();
        let parser = Parser::new("<style>@import 'file.css';</style>", &allocator, false);
        let root = parser.into_root();
        let css = root.css.unwrap();
        match &css.children[0] {
            StyleSheetChild::Atrule(at) => {
                assert_eq!(at.name, "import");
                assert!(at.block.is_none());
            }
            _ => panic!("expected Atrule"),
        }
    }
}
