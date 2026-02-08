use lux_ast::common::Span;
use lux_ast::template::attribute::{Attribute, AttributeNode, AttributeValue};
use winnow::Result;
use winnow::combinator::opt;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::{literal, take_while};

use crate::input::Input;
use crate::parser::utils::helpers::skip_whitespace;

use super::directive::parse_directive;
use super::is_attr_name_char;
use super::value::parse_attribute_value;

pub fn parse_named_attribute<'a>(input: &mut Input<'a>) -> Result<AttributeNode<'a>> {
    let attr_start = input.current_token_start();

    let name: &str = take_while(1.., is_attr_name_char).parse_next(input)?;

    if let Some(colon_pos) = name.find(':') {
        let prefix = &name[..colon_pos];
        let directive_name = &name[colon_pos + 1..];

        let (dir_name, modifiers_str) = split_modifiers(directive_name);

        skip_whitespace(input);

        let has_value = opt(literal("=")).parse_next(input)?.is_some();
        let value = if has_value {
            skip_whitespace(input);
            Some(parse_attribute_value(input)?)
        } else {
            None
        };

        let attr_end = input.previous_token_end();
        let span = Span::new(attr_start as u32, attr_end as u32);

        return parse_directive(input, prefix, dir_name, modifiers_str, value, span);
    }

    skip_whitespace(input);

    let value = if opt(literal("=")).parse_next(input)?.is_some() {
        skip_whitespace(input);
        parse_attribute_value(input)?
    } else {
        AttributeValue::True
    };

    let attr_end = input.previous_token_end();

    Ok(AttributeNode::Attribute(Attribute {
        span: Span::new(attr_start as u32, attr_end as u32),
        name,
        value,
    }))
}

fn split_modifiers(name: &str) -> (&str, Vec<&str>) {
    if let Some(pipe_pos) = name.find('|') {
        let base = &name[..pipe_pos];
        let mods = name[pipe_pos + 1..].split('|').collect();
        (base, mods)
    } else {
        (name, Vec::new())
    }
}
