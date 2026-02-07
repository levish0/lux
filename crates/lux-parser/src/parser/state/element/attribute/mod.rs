mod brace;
mod directive;
mod named;
mod value;

use lux_ast::template::attribute::AttributeNode;
use winnow::Result;

use crate::input::Input;
use crate::parser::utils::helpers::skip_whitespace;

pub use value::parse_attribute_value;

pub fn parse_attributes<'a>(input: &mut Input<'a>) -> Result<Vec<AttributeNode<'a>>> {
    let mut attrs = Vec::new();

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
            Ok(node) => attrs.push(node),
            Err(_) => break,
        }
    }

    Ok(attrs)
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
