use lux_ast::common::Span;
use lux_ast::template::attribute::{Attribute, AttributeValue};
use lux_ast::template::tag::Text;
use winnow::Result;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::take_while;

use crate::input::Input;
use crate::parser::utils::helpers::skip_whitespace;

use super::is_attr_name_char;

pub fn read_static_attributes<'a>(input: &mut Input<'a>) -> Result<Vec<Attribute<'a>>> {
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

        match read_static_attribute(input) {
            Ok(attr) => attrs.push(attr),
            Err(_) => break,
        }
    }

    Ok(attrs)
}

fn read_static_attribute<'a>(input: &mut Input<'a>) -> Result<Attribute<'a>> {
    let start = input.current_token_start();

    let name: &str = take_while(1.., is_attr_name_char).parse_next(input)?;

    skip_whitespace(input);

    let remaining: &str = &input.input;
    let value = if remaining.starts_with('=') {
        input.next_slice(1);
        skip_whitespace(input);
        read_static_value(input)?
    } else {
        AttributeValue::True
    };

    let end = input.previous_token_end();

    Ok(Attribute {
        span: Span::new(start as u32, end as u32),
        name,
        value,
        metadata: None,
    })
}

fn read_static_value<'a>(input: &mut Input<'a>) -> Result<AttributeValue<'a>> {
    let remaining: &str = &input.input;

    let (quote, content) = if remaining.starts_with('"') {
        input.next_slice(1);
        let val: &str = take_while(0.., |c: char| c != '"').parse_next(input)?;
        input.next_slice(1); // closing "
        ('"', val)
    } else if remaining.starts_with('\'') {
        input.next_slice(1);
        let val: &str = take_while(0.., |c: char| c != '\'').parse_next(input)?;
        input.next_slice(1); // closing '
        ('\'', val)
    } else {
        let val: &str =
            take_while(1.., |c: char| !c.is_ascii_whitespace() && c != '>' && c != '/')
                .parse_next(input)?;
        (' ', val)
    };

    let start = input.current_token_start() - content.len() - if quote != ' ' { 1 } else { 0 };
    let end = input.previous_token_end();

    Ok(AttributeValue::Sequence(vec![
        lux_ast::template::tag::TextOrExpressionTag::Text(Text {
            span: Span::new(start as u32, end as u32),
            data: content,
            raw: content,
        }),
    ]))
}
