use svelte_ast::css::*;
use svelte_ast::node::SimpleSelector;
use svelte_ast::span::Span;
use winnow::Result as ParseResult;

use super::{read_css_ident, skip_css_whitespace_and_comments};

/// Parse a selector list (comma-separated complex selectors).
pub fn parse_selector_list(
    source: &str,
    pos: &mut usize,
    offset: u32,
) -> ParseResult<SelectorList> {
    parse_selector_list_inner(source, pos, offset, false)
}

/// Inner selector list parser with inside_pseudo_class flag.
fn parse_selector_list_inner(
    source: &str,
    pos: &mut usize,
    offset: u32,
    inside_pseudo_class: bool,
) -> ParseResult<SelectorList> {
    // Skip leading whitespace and comments before first selector
    *pos = skip_css_whitespace_and_comments(source, *pos);
    let start = *pos;
    let mut children = Vec::new();

    children.push(parse_complex_selector(source, pos, offset, inside_pseudo_class)?);

    loop {
        let p = skip_css_whitespace_and_comments(source, *pos);
        if p >= source.len() || source.as_bytes()[p] != b',' {
            break;
        }
        *pos = p + 1; // consume comma
        *pos = skip_css_whitespace_and_comments(source, *pos);
        children.push(parse_complex_selector(source, pos, offset, inside_pseudo_class)?);
    }

    Ok(SelectorList {
        span: Span::new(start + offset as usize, *pos + offset as usize),
        children,
    })
}

/// Parse a complex selector (sequence of relative selectors with combinators).
fn parse_complex_selector(
    source: &str,
    pos: &mut usize,
    offset: u32,
    inside_pseudo_class: bool,
) -> ParseResult<ComplexSelector> {
    let start = *pos;
    let mut children = Vec::new();

    children.push(parse_relative_selector(source, pos, offset, inside_pseudo_class)?);

    loop {
        let ws_start = *pos;
        let p = skip_css_whitespace_and_comments(source, *pos);
        if p >= source.len() {
            break;
        }
        let ch = source.as_bytes()[p];
        if ch == b'{' || ch == b',' || ch == b')' {
            break;
        }

        let has_whitespace = p > ws_start;
        if ch == b'>' || ch == b'+' || ch == b'~' || ch == b'|' {
            let comb_name;
            let comb_start = p;
            if ch == b'|' && p + 1 < source.len() && source.as_bytes()[p + 1] == b'|' {
                comb_name = "||".to_string();
                *pos = p + 2;
            } else if ch == b'|' {
                break;
            } else {
                comb_name = String::from(ch as char);
                *pos = p + 1;
            }
            *pos = skip_css_whitespace_and_comments(source, *pos);

            let mut rel = parse_relative_selector(source, pos, offset, inside_pseudo_class)?;
            rel.combinator = Some(CssCombinator {
                span: Span::new(
                    comb_start + offset as usize,
                    comb_start + comb_name.len() + offset as usize,
                ),
                name: comb_name,
            });
            // RelativeSelector span starts at the combinator
            rel.span = Span::new(comb_start + offset as usize, rel.span.end);
            children.push(rel);
        } else if has_whitespace {
            *pos = p;
            let mut rel = parse_relative_selector(source, pos, offset, inside_pseudo_class)?;
            rel.combinator = Some(CssCombinator {
                span: Span::new(ws_start + offset as usize, p + offset as usize),
                name: " ".to_string(),
            });
            // RelativeSelector span starts at the combinator (whitespace start)
            rel.span = Span::new(ws_start + offset as usize, rel.span.end);
            children.push(rel);
        } else {
            break;
        }
    }

    Ok(ComplexSelector {
        span: Span::new(start + offset as usize, *pos + offset as usize),
        children,
    })
}

/// Parse a relative selector (sequence of simple selectors).
fn parse_relative_selector(
    source: &str,
    pos: &mut usize,
    offset: u32,
    inside_pseudo_class: bool,
) -> ParseResult<RelativeSelector> {
    let start = *pos;
    let mut selectors = Vec::new();

    loop {
        if *pos >= source.len() {
            break;
        }
        let ch = source.as_bytes()[*pos];
        if ch == b'{'
            || ch == b','
            || ch == b')'
            || ch.is_ascii_whitespace()
            || ch == b'>'
            || ch == b'~'
        {
            break;
        }
        if ch == b'+' {
            // Inside pseudo-class, '+' might start an nth expression like "+3n"
            if inside_pseudo_class && try_match_nth(&source[*pos..]).is_some() {
                // Let parse_simple_selector handle it as nth
            } else {
                break;
            }
        }

        selectors.push(parse_simple_selector(source, pos, offset, inside_pseudo_class)?);
    }

    if selectors.is_empty() {
        return Err(winnow::error::ContextError::new());
    }

    Ok(RelativeSelector {
        span: Span::new(start + offset as usize, *pos + offset as usize),
        combinator: None,
        selectors,
    })
}

/// Parse a single simple selector.
fn parse_simple_selector(
    source: &str,
    pos: &mut usize,
    offset: u32,
    inside_pseudo_class: bool,
) -> ParseResult<SimpleSelector> {
    let bytes = source.as_bytes();
    let start = *pos;

    match bytes[*pos] {
        b'.' => {
            *pos += 1;
            let name = read_css_ident(source, pos);
            Ok(SimpleSelector::ClassSelector(ClassSelector {
                span: Span::new(start + offset as usize, *pos + offset as usize),
                name,
            }))
        }
        b'#' => {
            *pos += 1;
            let name = read_css_ident(source, pos);
            Ok(SimpleSelector::IdSelector(IdSelector {
                span: Span::new(start + offset as usize, *pos + offset as usize),
                name,
            }))
        }
        b'[' => parse_attribute_selector(source, pos, offset),
        b':' => {
            if *pos + 1 < bytes.len() && bytes[*pos + 1] == b':' {
                parse_pseudo_element_selector(source, pos, offset)
            } else {
                parse_pseudo_class_selector(source, pos, offset)
            }
        }
        b'&' => {
            *pos += 1;
            Ok(SimpleSelector::NestingSelector(NestingSelector {
                span: Span::new(start + offset as usize, *pos + offset as usize),
                name: "&".to_string(),
            }))
        }
        b'*' => {
            *pos += 1;
            Ok(SimpleSelector::TypeSelector(TypeSelector {
                span: Span::new(start + offset as usize, *pos + offset as usize),
                name: "*".to_string(),
            }))
        }
        _ => {
            // When inside a pseudo-class, try nth pattern first
            if inside_pseudo_class {
                if let Some(nth_len) = try_match_nth(&source[*pos..]) {
                    let value = source[*pos..*pos + nth_len].to_string();
                    let nth_start = *pos;
                    *pos += nth_len;
                    return Ok(SimpleSelector::Nth(Nth {
                        span: Span::new(nth_start + offset as usize, *pos + offset as usize),
                        value,
                    }));
                }
            }
            // Type selector (element name)
            let name = read_css_ident(source, pos);
            if name.is_empty() {
                return Err(winnow::error::ContextError::new());
            }
            Ok(SimpleSelector::TypeSelector(TypeSelector {
                span: Span::new(start + offset as usize, *pos + offset as usize),
                name,
            }))
        }
    }
}

/// Try to match an nth expression pattern at the start of the string.
/// Returns the length of the matched portion, or None if no match.
/// Matches: even, odd, [+-]?\d*n?(\s*[+-]\s*\d+)? followed by end/,/)/ or " of ".
/// For "of" case, the returned length includes " of " and trailing whitespace.
fn try_match_nth(source: &str) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut p: usize = 0;

    // Match the An+B part
    if bytes.len() >= 4 && &source[..4] == "even" && (bytes.len() == 4 || !bytes[4].is_ascii_alphanumeric()) {
        p = 4;
    } else if bytes.len() >= 3 && &source[..3] == "odd" && (bytes.len() == 3 || !bytes[3].is_ascii_alphanumeric()) {
        p = 3;
    } else {
        // [+-]?\d*n?(\s*[+-]\s*\d+)?
        if p < bytes.len() && (bytes[p] == b'+' || bytes[p] == b'-') {
            p += 1;
        }
        let digit_start = p;
        while p < bytes.len() && bytes[p].is_ascii_digit() {
            p += 1;
        }
        let has_n = p < bytes.len() && (bytes[p] == b'n' || bytes[p] == b'N');
        if has_n {
            p += 1;
        }
        // Must have consumed digits or 'n'
        if !has_n && p == digit_start {
            return None;
        }
        // Optional [+-]\d+ suffix (only valid after 'n')
        if has_n {
            let mut q = p;
            while q < bytes.len() && bytes[q].is_ascii_whitespace() {
                q += 1;
            }
            if q < bytes.len() && (bytes[q] == b'+' || bytes[q] == b'-') {
                q += 1;
                while q < bytes.len() && bytes[q].is_ascii_whitespace() {
                    q += 1;
                }
                let ds = q;
                while q < bytes.len() && bytes[q].is_ascii_digit() {
                    q += 1;
                }
                if q > ds {
                    p = q;
                }
            }
        }
    }

    if p == 0 {
        return None;
    }

    // Check terminator
    let mut q = p;
    while q < bytes.len() && bytes[q].is_ascii_whitespace() {
        q += 1;
    }
    // End of input, comma, or close paren → valid
    if q >= bytes.len() || bytes[q] == b',' || bytes[q] == b')' {
        return Some(p);
    }

    // Check for " of " keyword (needs whitespace before "of")
    if q > p && q + 2 <= bytes.len() && &source[q..q + 2] == "of" {
        let after_of = q + 2;
        if after_of < bytes.len() && bytes[after_of].is_ascii_whitespace() {
            // Include "of" and trailing whitespace in the value
            let mut r = after_of;
            while r < bytes.len() && bytes[r].is_ascii_whitespace() {
                r += 1;
            }
            return Some(r);
        }
    }

    None
}

/// Parse an attribute selector: [name], [name=value], [name~=value i], etc.
fn parse_attribute_selector(
    source: &str,
    pos: &mut usize,
    offset: u32,
) -> ParseResult<SimpleSelector> {
    let start = *pos;
    *pos += 1; // consume [

    let p = skip_css_whitespace_and_comments(source, *pos);
    *pos = p;

    let name = read_css_ident(source, pos);

    *pos = skip_css_whitespace_and_comments(source, *pos);

    let mut matcher = None;
    let mut value = None;
    let mut flags = None;

    if *pos < source.len() && source.as_bytes()[*pos] != b']' {
        let bytes = source.as_bytes();
        if bytes[*pos] == b'=' {
            matcher = Some("=".to_string());
            *pos += 1;
        } else if *pos + 1 < bytes.len() && bytes[*pos + 1] == b'=' {
            matcher = Some(format!("{}=", bytes[*pos] as char));
            *pos += 2;
        }

        if matcher.is_some() {
            *pos = skip_css_whitespace_and_comments(source, *pos);
            if *pos < source.len() {
                value = Some(read_attribute_value(source, pos).to_string());
            }

            *pos = skip_css_whitespace_and_comments(source, *pos);
            if *pos < source.len() {
                let ch = source.as_bytes()[*pos];
                if ch == b'i' || ch == b's' || ch == b'I' || ch == b'S' {
                    flags = Some(String::from(ch as char));
                    *pos += 1;
                }
            }
        }
    }

    *pos = skip_css_whitespace_and_comments(source, *pos);
    if *pos < source.len() && source.as_bytes()[*pos] == b']' {
        *pos += 1;
    }

    Ok(SimpleSelector::AttributeSelector(AttributeSelector {
        span: Span::new(start + offset as usize, *pos + offset as usize),
        name,
        matcher,
        value,
        flags,
    }))
}

/// Parse a pseudo-class selector: :name or :name(args)
fn parse_pseudo_class_selector(
    source: &str,
    pos: &mut usize,
    offset: u32,
) -> ParseResult<SimpleSelector> {
    let start = *pos;
    *pos += 1; // consume :

    let name = read_css_ident(source, pos);

    let args = if *pos < source.len() && source.as_bytes()[*pos] == b'(' {
        let paren_pos = *pos;
        *pos += 1; // consume (
        let args_content = read_balanced_parens_content(source, pos);
        if args_content.trim().is_empty() {
            None
        } else {
            // Offset of first char after ( in the overall document
            let inner_offset = offset + (paren_pos as u32) + 1;

            // Parse as selector list with inside_pseudo_class=true (enables nth detection)
            // parse_selector_list_inner will skip leading whitespace/comments
            let mut inner_pos = 0;
            match parse_selector_list_inner(
                args_content,
                &mut inner_pos,
                inner_offset,
                true,
            ) {
                Ok(list) => {
                    // Check all content consumed (allow trailing whitespace/comments)
                    let remaining =
                        skip_css_whitespace_and_comments(args_content, inner_pos);
                    if remaining >= args_content.len() {
                        Some(Box::new(list))
                    } else {
                        // Fallback: wrap entire content as Nth
                        let trimmed = args_content.trim();
                        let leading =
                            args_content.len() - args_content.trim_start().len();
                        let nth_start = inner_offset as usize + leading;
                        let nth_end = nth_start + trimmed.len();
                        let nth_span = Span::new(nth_start, nth_end);
                        let nth = SimpleSelector::Nth(Nth {
                            span: nth_span.clone(),
                            value: trimmed.to_string(),
                        });
                        let rel = RelativeSelector {
                            span: nth_span.clone(),
                            combinator: None,
                            selectors: vec![nth],
                        };
                        let complex = ComplexSelector {
                            span: nth_span.clone(),
                            children: vec![rel],
                        };
                        Some(Box::new(SelectorList {
                            span: nth_span,
                            children: vec![complex],
                        }))
                    }
                }
                Err(_) => {
                    // Fallback: wrap entire content as Nth
                    let trimmed = args_content.trim();
                    let leading =
                        args_content.len() - args_content.trim_start().len();
                    let nth_start = inner_offset as usize + leading;
                    let nth_end = nth_start + trimmed.len();
                    let nth_span = Span::new(nth_start, nth_end);
                    let nth = SimpleSelector::Nth(Nth {
                        span: nth_span.clone(),
                        value: trimmed.to_string(),
                    });
                    let rel = RelativeSelector {
                        span: nth_span.clone(),
                        combinator: None,
                        selectors: vec![nth],
                    };
                    let complex = ComplexSelector {
                        span: nth_span.clone(),
                        children: vec![rel],
                    };
                    Some(Box::new(SelectorList {
                        span: nth_span,
                        children: vec![complex],
                    }))
                }
            }
        }
    } else {
        None
    };

    Ok(SimpleSelector::PseudoClassSelector(PseudoClassSelector {
        span: Span::new(start + offset as usize, *pos + offset as usize),
        name,
        args,
    }))
}

/// Parse a pseudo-element selector: ::name or ::name(args)
fn parse_pseudo_element_selector(
    source: &str,
    pos: &mut usize,
    offset: u32,
) -> ParseResult<SimpleSelector> {
    let start = *pos;
    *pos += 2; // consume ::

    let name = read_css_ident(source, pos);
    let name_end = *pos; // span ends at name, not after args

    // Some pseudo-elements take arguments: ::slotted(.foo), ::part(bar), ::highlight(x)
    if *pos < source.len() && source.as_bytes()[*pos] == b'(' {
        *pos += 1;
        let _args = read_balanced_parens_content(source, pos);
    }

    Ok(SimpleSelector::PseudoElementSelector(
        PseudoElementSelector {
            span: Span::new(start + offset as usize, name_end + offset as usize),
            name,
        },
    ))
}

/// Read content between balanced parentheses.
/// Assumes the opening ( has already been consumed.
/// Consumes the closing ).
fn read_balanced_parens_content<'a>(source: &'a str, pos: &mut usize) -> &'a str {
    let mut depth: u32 = 1;
    let start = *pos;
    let bytes = source.as_bytes();

    while *pos < bytes.len() && depth > 0 {
        match bytes[*pos] {
            b'(' => {
                depth += 1;
                *pos += 1;
            }
            b')' => {
                depth -= 1;
                if depth == 0 {
                    let content = &source[start..*pos];
                    *pos += 1; // consume closing )
                    return content;
                }
                *pos += 1;
            }
            b'/' if *pos + 1 < bytes.len() && bytes[*pos + 1] == b'*' => {
                // Block comment - skip until */
                *pos += 2;
                while *pos + 1 < bytes.len() {
                    if bytes[*pos] == b'*' && bytes[*pos + 1] == b'/' {
                        *pos += 2;
                        break;
                    }
                    *pos += 1;
                }
            }
            b'\'' | b'"' => {
                let quote = bytes[*pos];
                *pos += 1;
                while *pos < bytes.len() && bytes[*pos] != quote {
                    if bytes[*pos] == b'\\' {
                        *pos += 1;
                        if *pos >= bytes.len() {
                            break;
                        }
                    }
                    *pos += 1;
                }
                if *pos < bytes.len() {
                    *pos += 1; // consume closing quote
                }
            }
            _ => {
                *pos += 1;
            }
        }
    }
    let end = (*pos).min(source.len());
    &source[start..end]
}

/// Read an attribute value (quoted or unquoted).
fn read_attribute_value<'a>(source: &'a str, pos: &mut usize) -> &'a str {
    let bytes = source.as_bytes();
    if *pos >= bytes.len() {
        return "";
    }

    let ch = bytes[*pos];
    if ch == b'"' || ch == b'\'' {
        let quote = ch;
        *pos += 1;
        let start = *pos;
        while *pos < bytes.len() && bytes[*pos] != quote {
            if bytes[*pos] == b'\\' {
                *pos += 1;
                if *pos >= bytes.len() {
                    break;
                }
            }
            *pos += 1;
        }
        let value = &source[start..*pos];
        if *pos < bytes.len() {
            *pos += 1; // consume closing quote
        }
        value
    } else {
        let start = *pos;
        while *pos < bytes.len() && !bytes[*pos].is_ascii_whitespace() && bytes[*pos] != b']' {
            *pos += 1;
        }
        &source[start..*pos]
    }
}
