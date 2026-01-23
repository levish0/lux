use svelte_ast::css::*;
use svelte_ast::node::SimpleSelector;
use svelte_ast::span::Span;
use winnow::Result as ParseResult;

use super::skip_css_whitespace_and_comments;

/// Parse a selector list (comma-separated complex selectors).
pub fn parse_selector_list(
    source: &str,
    pos: &mut usize,
    offset: u32,
) -> ParseResult<SelectorList> {
    let start = *pos;
    let mut children = Vec::new();

    children.push(parse_complex_selector(source, pos, offset)?);

    loop {
        let p = skip_css_whitespace_and_comments(source, *pos);
        if p >= source.len() || source.as_bytes()[p] != b',' {
            break;
        }
        *pos = p + 1; // consume comma
        *pos = skip_css_whitespace_and_comments(source, *pos);
        children.push(parse_complex_selector(source, pos, offset)?);
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
) -> ParseResult<ComplexSelector> {
    let start = *pos;
    let mut children = Vec::new();

    // First relative selector (no combinator)
    children.push(parse_relative_selector(source, pos, offset, true)?);

    // Subsequent relative selectors with combinators
    loop {
        let ws_start = *pos;
        let p = skip_css_whitespace_and_comments(source, *pos);
        if p >= source.len() {
            break;
        }
        let ch = source.as_bytes()[p];
        // Stop at block start, comma, or closing paren
        if ch == b'{' || ch == b',' || ch == b')' {
            break;
        }

        // Check for explicit combinator
        let has_whitespace = p > ws_start;
        if ch == b'>' || ch == b'+' || ch == b'~' || ch == b'|' {
            // Handle || combinator
            let comb_name;
            let comb_start = p;
            if ch == b'|' && p + 1 < source.len() && source.as_bytes()[p + 1] == b'|' {
                comb_name = "||".to_string();
                *pos = p + 2;
            } else if ch == b'|' {
                // Not a combinator, just a pipe in selector
                break;
            } else {
                comb_name = String::from(ch as char);
                *pos = p + 1;
            }
            *pos = skip_css_whitespace_and_comments(source, *pos);

            let mut rel = parse_relative_selector(source, pos, offset, false)?;
            rel.combinator = Some(CssCombinator {
                span: Span::new(
                    comb_start + offset as usize,
                    comb_start + comb_name.len() + offset as usize,
                ),
                name: comb_name,
            });
            children.push(rel);
        } else if has_whitespace {
            // Descendant combinator (whitespace)
            *pos = p;
            let mut rel = parse_relative_selector(source, pos, offset, false)?;
            rel.combinator = Some(CssCombinator {
                span: Span::new(ws_start + offset as usize, p + offset as usize),
                name: " ".to_string(),
            });
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
    _is_first: bool,
) -> ParseResult<RelativeSelector> {
    let start = *pos;
    let mut selectors = Vec::new();

    loop {
        if *pos >= source.len() {
            break;
        }
        let ch = source.as_bytes()[*pos];
        // Stop conditions
        if ch == b'{'
            || ch == b','
            || ch == b')'
            || ch.is_ascii_whitespace()
            || ch == b'>'
            || ch == b'+'
            || ch == b'~'
        {
            break;
        }

        selectors.push(parse_simple_selector(source, pos, offset)?);
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
) -> ParseResult<SimpleSelector> {
    let bytes = source.as_bytes();
    let start = *pos;

    match bytes[*pos] {
        b'.' => {
            // Class selector
            *pos += 1;
            let name = read_ident(source, pos);
            Ok(SimpleSelector::ClassSelector(ClassSelector {
                span: Span::new(start + offset as usize, *pos + offset as usize),
                name,
            }))
        }
        b'#' => {
            // ID selector
            *pos += 1;
            let name = read_ident(source, pos);
            Ok(SimpleSelector::IdSelector(IdSelector {
                span: Span::new(start + offset as usize, *pos + offset as usize),
                name,
            }))
        }
        b'[' => {
            // Attribute selector
            parse_attribute_selector(source, pos, offset)
        }
        b':' => {
            if *pos + 1 < bytes.len() && bytes[*pos + 1] == b':' {
                // Pseudo-element selector
                parse_pseudo_element_selector(source, pos, offset)
            } else {
                // Pseudo-class selector
                parse_pseudo_class_selector(source, pos, offset)
            }
        }
        b'&' => {
            // Nesting selector
            *pos += 1;
            Ok(SimpleSelector::NestingSelector(NestingSelector {
                span: Span::new(start + offset as usize, *pos + offset as usize),
                name: "&".to_string(),
            }))
        }
        b'*' => {
            // Universal selector
            *pos += 1;
            Ok(SimpleSelector::TypeSelector(TypeSelector {
                span: Span::new(start + offset as usize, *pos + offset as usize),
                name: "*".to_string(),
            }))
        }
        _ => {
            // Type selector (element name)
            let name = read_ident(source, pos);
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

    let name = read_ident(source, pos);

    *pos = skip_css_whitespace_and_comments(source, *pos);

    let mut matcher = None;
    let mut value = None;
    let mut flags = None;

    if *pos < source.len() && source.as_bytes()[*pos] != b']' {
        // Read matcher (=, ~=, |=, ^=, $=, *=)
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
            // Read value (quoted or unquoted)
            if *pos < source.len() {
                value = Some(read_attribute_value(source, pos));
            }

            *pos = skip_css_whitespace_and_comments(source, *pos);
            // Check for flags (i, s)
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

    let name = read_ident(source, pos);

    let args = if *pos < source.len() && source.as_bytes()[*pos] == b'(' {
        *pos += 1; // consume (
        // For :nth-child, :nth-of-type etc., the args might contain An+B syntax
        // or nested selectors. We'll parse it as a selector list if possible,
        // otherwise as a raw Nth/Percentage value.
        let args_content = read_balanced_parens_content(source, pos);
        if args_content.trim().is_empty() {
            None
        } else {
            // Try to parse as selector list
            let mut inner_pos = 0;
            let inner_source = args_content.trim();
            match parse_selector_list(
                inner_source,
                &mut inner_pos,
                offset + (start as u32) + 1 + name.len() as u32 + 1,
            ) {
                Ok(list) if inner_pos >= inner_source.len() => Some(Box::new(list)),
                _ => {
                    // Treat as Nth syntax - wrap in a single pseudo selector with Nth child
                    let nth_span = Span::new(
                        start + offset as usize + 1 + name.len() + 1,
                        *pos + offset as usize - 1,
                    );
                    let nth = SimpleSelector::Nth(Nth {
                        span: nth_span.clone(),
                        value: args_content.trim().to_string(),
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

    let name = read_ident(source, pos);

    // Some pseudo-elements take arguments: ::slotted(.foo), ::part(bar), ::highlight(x)
    if *pos < source.len() && source.as_bytes()[*pos] == b'(' {
        *pos += 1;
        // Read and discard args for pseudo-elements (they're part of the name in the span)
        let _args = read_balanced_parens_content(source, pos);
    }

    Ok(SimpleSelector::PseudoElementSelector(
        PseudoElementSelector {
            span: Span::new(start + offset as usize, *pos + offset as usize),
            name,
        },
    ))
}

/// Read content between balanced parentheses.
/// Assumes the opening ( has already been consumed.
/// Consumes the closing ).
fn read_balanced_parens_content(source: &str, pos: &mut usize) -> String {
    let mut depth = 1;
    let start = *pos;
    let bytes = source.as_bytes();

    while *pos < bytes.len() && depth > 0 {
        match bytes[*pos] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    let content = source[start..*pos].to_string();
                    *pos += 1; // consume closing )
                    return content;
                }
            }
            b'\'' | b'"' => {
                // Skip quoted strings
                let quote = bytes[*pos];
                *pos += 1;
                while *pos < bytes.len() && bytes[*pos] != quote {
                    if bytes[*pos] == b'\\' {
                        *pos += 1;
                    }
                    *pos += 1;
                }
                // pos now at closing quote
            }
            _ => {}
        }
        *pos += 1;
    }
    source[start..*pos].to_string()
}

/// Read a CSS identifier.
fn read_ident(source: &str, pos: &mut usize) -> String {
    let start = *pos;
    let bytes = source.as_bytes();

    // CSS idents can start with -, _, or a letter
    while *pos < bytes.len() {
        let ch = bytes[*pos];
        if ch.is_ascii_alphanumeric() || ch == b'-' || ch == b'_' {
            *pos += 1;
        } else if ch > 127 {
            // Non-ASCII chars (unicode idents)
            *pos += 1;
        } else {
            break;
        }
    }
    source[start..*pos].to_string()
}

/// Read an attribute value (quoted or unquoted).
fn read_attribute_value(source: &str, pos: &mut usize) -> String {
    let bytes = source.as_bytes();
    if *pos >= bytes.len() {
        return String::new();
    }

    let ch = bytes[*pos];
    if ch == b'"' || ch == b'\'' {
        // Quoted value
        let quote = ch;
        *pos += 1;
        let start = *pos;
        while *pos < bytes.len() && bytes[*pos] != quote {
            if bytes[*pos] == b'\\' {
                *pos += 1;
            }
            *pos += 1;
        }
        let value = source[start..*pos].to_string();
        if *pos < bytes.len() {
            *pos += 1; // consume closing quote
        }
        value
    } else {
        // Unquoted value
        let start = *pos;
        while *pos < bytes.len() && !bytes[*pos].is_ascii_whitespace() && bytes[*pos] != b']' {
            *pos += 1;
        }
        source[start..*pos].to_string()
    }
}
