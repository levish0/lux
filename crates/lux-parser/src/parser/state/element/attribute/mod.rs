mod brace;
mod directive;
mod named;
pub mod static_attr;
mod value;

use lux_ast::template::attribute::AttributeNode;
use rustc_hash::FxHashSet;
use winnow::Result;

use crate::error::{ErrorKind, ParseError};
use crate::input::Input;
use crate::parser::utils::helpers::skip_whitespace;

pub use value::parse_attribute_value;

pub fn parse_attributes<'a>(input: &mut Input<'a>) -> Result<Vec<AttributeNode<'a>>> {
    let mut attrs = Vec::new();
    let mut seen = FxHashSet::default();

    loop {
        skip_whitespace(input);

        let remaining: &str = &input.input;
        if remaining.is_empty() {
            break;
        }

        let first = remaining.as_bytes()[0];

        if first == b'>' || first == b'/' {
            break;
        }

        if first == b'{' {
            let node = brace::parse_brace_attribute(input)?;
            attrs.push(node);
            continue;
        }

        match named::parse_named_attribute(input) {
            Ok(node) => {
                check_duplicate(input, &node, &mut seen);
                attrs.push(node);
            }
            Err(_) => break,
        }
    }

    Ok(attrs)
}

fn check_duplicate(input: &mut Input<'_>, node: &AttributeNode<'_>, seen: &mut FxHashSet<String>) {
    let (kind, name, span) = match node {
        AttributeNode::Attribute(a) => ("Attribute", a.name, a.span),
        AttributeNode::BindDirective(b) => ("Attribute", b.name, b.span),
        AttributeNode::ClassDirective(c) => ("ClassDirective", c.name, c.span),
        AttributeNode::StyleDirective(s) => ("StyleDirective", s.name, s.span),
        _ => return,
    };

    if name == "this" {
        return;
    }

    let key = format!("{kind}-{name}");
    if !seen.insert(key) {
        input.state.errors.push(ParseError::new(
            ErrorKind::DuplicateAttribute,
            span,
            format!("Duplicate attribute '{name}'"),
        ));
    }
}

pub fn is_attr_name_char(c: char) -> bool {
    !c.is_ascii_whitespace()
        && c != '='
        && c != '>'
        && c != '/'
        && c != '"'
        && c != '\''
        && c != '{'
        && c != '}'
}

pub fn is_tag_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ':' || c == '.'
}
