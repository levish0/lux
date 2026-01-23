use svelte_ast::css::*;
use svelte_ast::node::{CssBlockChild, StyleSheetChild};
use svelte_ast::span::Span;
use winnow::Result as ParseResult;

use super::selectors::parse_selector_list;
use super::skip_css_whitespace_and_comments;

/// Parse a single stylesheet child (Rule or Atrule).
pub fn css_child_parser(source: &str, pos: &mut usize, offset: u32) -> ParseResult<StyleSheetChild> {
    let p = skip_css_whitespace_and_comments(source, *pos);
    *pos = p;

    if *pos >= source.len() {
        return Err(winnow::error::ContextError::new());
    }

    if source.as_bytes()[*pos] == b'@' {
        let atrule = parse_atrule(source, pos, offset)?;
        Ok(StyleSheetChild::Atrule(atrule))
    } else {
        let rule = parse_rule(source, pos, offset)?;
        Ok(StyleSheetChild::Rule(rule))
    }
}

/// Parse a CSS rule: selector { declarations }
fn parse_rule(source: &str, pos: &mut usize, offset: u32) -> ParseResult<CssRule> {
    let start = *pos;

    // Parse selector list (stops at {)
    let prelude = parse_selector_list(source, pos, offset)?;

    // Skip to block
    *pos = skip_css_whitespace_and_comments(source, *pos);

    // Parse block
    let block = parse_block(source, pos, offset)?;

    Ok(CssRule {
        span: Span::new(start + offset as usize, *pos + offset as usize),
        prelude,
        block,
    })
}

/// Parse an at-rule: @name prelude { block } or @name prelude;
fn parse_atrule(source: &str, pos: &mut usize, offset: u32) -> ParseResult<CssAtrule> {
    let start = *pos;
    *pos += 1; // consume @

    // Read at-rule name
    let name = read_css_ident(source, pos);

    // Read prelude (everything up to { or ;)
    *pos = skip_css_whitespace_and_comments(source, *pos);
    let prelude_start = *pos;
    let bytes = source.as_bytes();

    let mut depth = 0;
    while *pos < bytes.len() {
        match bytes[*pos] {
            b'{' if depth == 0 => break,
            b'(' => { depth += 1; *pos += 1; }
            b')' => { depth -= 1; *pos += 1; }
            b';' if depth == 0 => break,
            b'"' | b'\'' => {
                let quote = bytes[*pos];
                *pos += 1;
                while *pos < bytes.len() && bytes[*pos] != quote {
                    if bytes[*pos] == b'\\' { *pos += 1; }
                    *pos += 1;
                }
                if *pos < bytes.len() { *pos += 1; }
            }
            _ => { *pos += 1; }
        }
    }
    let prelude = source[prelude_start..*pos].trim().to_string();

    let block = if *pos < bytes.len() && bytes[*pos] == b'{' {
        Some(parse_block(source, pos, offset)?)
    } else {
        if *pos < bytes.len() && bytes[*pos] == b';' {
            *pos += 1;
        }
        None
    };

    Ok(CssAtrule {
        span: Span::new(start + offset as usize, *pos + offset as usize),
        name,
        prelude,
        block,
    })
}

/// Parse a CSS block: { declarations/rules }
fn parse_block(source: &str, pos: &mut usize, offset: u32) -> ParseResult<CssBlock> {
    let start = *pos;
    let bytes = source.as_bytes();

    if *pos >= bytes.len() || bytes[*pos] != b'{' {
        return Err(winnow::error::ContextError::new());
    }
    *pos += 1; // consume {

    let mut children = Vec::new();

    loop {
        *pos = skip_css_whitespace_and_comments(source, *pos);
        if *pos >= bytes.len() {
            break;
        }
        if bytes[*pos] == b'}' {
            *pos += 1; // consume }
            break;
        }

        // Determine if this is a nested rule, at-rule, or declaration
        if bytes[*pos] == b'@' {
            let atrule = parse_atrule(source, pos, offset)?;
            children.push(CssBlockChild::Atrule(atrule));
        } else if is_nested_rule(source, *pos) {
            let rule = parse_rule(source, pos, offset)?;
            children.push(CssBlockChild::Rule(rule));
        } else {
            let decl = parse_declaration(source, pos, offset)?;
            children.push(CssBlockChild::Declaration(decl));
        }
    }

    Ok(CssBlock {
        span: Span::new(start + offset as usize, *pos + offset as usize),
        children,
    })
}

/// Determine if the current position starts a nested rule (not a declaration).
/// Scans forward: if `{` is encountered before `;` or `}` (at depth 0), it's a rule.
/// Otherwise it's a declaration.
fn is_nested_rule(source: &str, pos: usize) -> bool {
    let bytes = source.as_bytes();
    let mut p = pos;
    let mut depth = 0;

    while p < bytes.len() {
        match bytes[p] {
            b'(' | b'[' => { depth += 1; }
            b')' | b']' => { if depth > 0 { depth -= 1; } }
            b'{' if depth == 0 => return true,
            b';' if depth == 0 => return false,
            b'}' if depth == 0 => return false,
            b'"' | b'\'' => {
                // Skip quoted strings
                let quote = bytes[p];
                p += 1;
                while p < bytes.len() && bytes[p] != quote {
                    if bytes[p] == b'\\' { p += 1; }
                    p += 1;
                }
            }
            _ => {}
        }
        p += 1;
    }
    false
}

/// Parse a CSS declaration: property: value;
fn parse_declaration(source: &str, pos: &mut usize, offset: u32) -> ParseResult<CssDeclaration> {
    let start = *pos;
    let bytes = source.as_bytes();

    // Read property name
    let property = read_declaration_property(source, pos);

    // Skip whitespace and consume :
    *pos = skip_css_whitespace_and_comments(source, *pos);
    if *pos < bytes.len() && bytes[*pos] == b':' {
        *pos += 1;
    }
    *pos = skip_css_whitespace_and_comments(source, *pos);

    // Read value (until ; or })
    let value_start = *pos;
    let mut depth = 0;
    while *pos < bytes.len() {
        match bytes[*pos] {
            b';' if depth == 0 => {
                break;
            }
            b'}' if depth == 0 => {
                break;
            }
            b'(' => { depth += 1; *pos += 1; }
            b')' => { depth -= 1; *pos += 1; }
            b'"' | b'\'' => {
                let quote = bytes[*pos];
                *pos += 1;
                while *pos < bytes.len() && bytes[*pos] != quote {
                    if bytes[*pos] == b'\\' { *pos += 1; }
                    *pos += 1;
                }
                if *pos < bytes.len() { *pos += 1; }
            }
            _ => { *pos += 1; }
        }
    }
    let value = source[value_start..*pos].trim().to_string();
    let end = *pos;

    // Consume optional ;
    if *pos < bytes.len() && bytes[*pos] == b';' {
        *pos += 1;
    }

    Ok(CssDeclaration {
        span: Span::new(start + offset as usize, end + offset as usize),
        property,
        value,
    })
}

/// Read a declaration property name.
fn read_declaration_property(source: &str, pos: &mut usize) -> String {
    let start = *pos;
    let bytes = source.as_bytes();

    // CSS custom properties can start with --
    while *pos < bytes.len() {
        let ch = bytes[*pos];
        if ch.is_ascii_alphanumeric() || ch == b'-' || ch == b'_' {
            *pos += 1;
        } else {
            break;
        }
    }
    source[start..*pos].to_string()
}

/// Read a CSS identifier.
fn read_css_ident(source: &str, pos: &mut usize) -> String {
    let start = *pos;
    let bytes = source.as_bytes();

    while *pos < bytes.len() {
        let ch = bytes[*pos];
        if ch.is_ascii_alphanumeric() || ch == b'-' || ch == b'_' {
            *pos += 1;
        } else if ch > 127 {
            *pos += 1;
        } else {
            break;
        }
    }
    source[start..*pos].to_string()
}
