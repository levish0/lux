mod close;
mod next;
mod open;
mod special;

use oxc_ast::ast::{BindingPattern, Expression};
use oxc_span::GetSpan;

use lux_ast::metadata::ExpressionNodeMetadata;
use lux_ast::node::FragmentNode;
use lux_ast::span::Span;
use lux_ast::tags::ExpressionTag;

use crate::parser::read::expression::read_expression;
use crate::parser::{ParseError, Parser};

/// Tag state.
/// Matches reference: `state/tag.js`
///
/// Handles `{...}` expressions and blocks (`{#if}`, `{:else}`, `{/if}`, `{@html}`, etc.)
pub fn tag(parser: &mut Parser) -> Result<(), ParseError> {
    let start = parser.index;
    parser.index += 1; // skip `{`

    parser.allow_whitespace();

    if parser.eat("#") {
        return open::open(parser);
    }
    if parser.eat(":") {
        return next::next(parser);
    }
    if parser.eat("@") {
        return special::special(parser);
    }
    if parser.match_str("/") && !parser.match_str("/*") && !parser.match_str("//") {
        parser.eat("/");
        return close::close(parser);
    }

    // Expression tag: {expression}
    let expression = read_expression(parser)?;

    parser.allow_whitespace();
    parser.eat_required("}")?;

    parser.append(FragmentNode::ExpressionTag(ExpressionTag {
        span: Span::new(start, parser.index),
        expression,
        metadata: ExpressionNodeMetadata::default(),
    }));
    Ok(())
}

// ─── Shared Helper Functions ────────────────────────────────────────

/// Skip to the matching closing `}`, handling nested braces.
pub fn skip_to_closing_brace(parser: &mut Parser) {
    let mut depth = 1u32;
    while parser.index < parser.template.len() && depth > 0 {
        let ch = parser.template.as_bytes()[parser.index];
        match ch {
            b'{' => depth += 1,
            b'}' => depth -= 1,
            b'\'' | b'"' | b'`' => {
                skip_string(parser, ch);
                if depth > 0 {
                    continue;
                }
            }
            _ => {}
        }
        if depth > 0 {
            parser.index += 1;
        }
    }
    if depth == 0 {
        parser.index += 1; // skip closing `}`
    }
}

/// Skip a string literal (single, double, or template).
fn skip_string(parser: &mut Parser, quote: u8) {
    parser.index += 1; // skip opening quote
    while parser.index < parser.template.len() {
        let ch = parser.template.as_bytes()[parser.index];
        if ch == b'\\' {
            parser.index += 1; // skip escape
        } else if ch == quote {
            return; // don't skip closing quote — caller will advance
        } else if quote == b'`' && ch == b'$' {
            if parser.index + 1 < parser.template.len()
                && parser.template.as_bytes()[parser.index + 1] == b'{'
            {
                parser.index += 2;
                skip_to_closing_brace(parser);
                continue;
            }
        }
        parser.index += 1;
    }
}

/// Get span from a BindingPattern.
pub fn binding_pattern_span(pattern: &BindingPattern) -> oxc_span::Span {
    match pattern {
        BindingPattern::BindingIdentifier(id) => id.span,
        BindingPattern::ObjectPattern(p) => p.span,
        BindingPattern::ArrayPattern(p) => p.span,
        BindingPattern::AssignmentPattern(p) => p.span,
    }
}

/// Get the end position of an expression.
pub fn expression_end(expr: &Expression) -> u32 {
    expr.span().end
}

/// Skip a string literal in byte scanning. Returns the position after the closing quote.
pub fn skip_string_bytes(bytes: &[u8], start: usize) -> usize {
    let quote = bytes[start];
    let mut i = start + 1;
    while i < bytes.len() {
        let ch = bytes[i];
        if ch == b'\\' {
            i += 2;
            continue;
        }
        if ch == quote {
            return i + 1;
        }
        if quote == b'`' && ch == b'$' && i + 1 < bytes.len() && bytes[i + 1] == b'{' {
            i += 2;
            let mut depth = 1u32;
            while i < bytes.len() && depth > 0 {
                match bytes[i] {
                    b'{' => depth += 1,
                    b'}' => depth -= 1,
                    b'\\' => {
                        i += 1;
                    }
                    _ => {}
                }
                i += 1;
            }
            continue;
        }
        i += 1;
    }
    i
}

pub fn is_whitespace_byte(b: u8) -> bool {
    b == b' ' || b == b'\t' || b == b'\r' || b == b'\n'
}

pub fn is_identifier_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}
