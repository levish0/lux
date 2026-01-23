use svelte_ast::attributes::{Attribute, AttributeSequenceValue, AttributeValue};
use svelte_ast::node::AttributeNode;
use svelte_ast::span::Span;
use svelte_ast::text::Text;
use winnow::combinator::opt;
use winnow::prelude::*;
use winnow::stream::Location;
use winnow::token::{literal, take_while};
use winnow::Result as ParseResult;

use super::ParserInput;

pub fn attribute_parser(parser_input: &mut ParserInput) -> ParseResult<AttributeNode> {
    let start = parser_input.input.current_token_start();

    let name_start = parser_input.input.current_token_start();
    let name: &str = take_while(1.., |c: char| {
        c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | ':' | '.')
    })
    .parse_next(parser_input)?;
    let name_end = parser_input.input.previous_token_end();
    let name_loc = Span::new(name_start, name_end);

    // Check for = sign
    let has_value = opt(literal("=")).parse_next(parser_input)?.is_some();

    let value = if has_value {
        parse_attribute_value(parser_input)?
    } else {
        AttributeValue::True
    };

    let end = parser_input.input.previous_token_end();

    Ok(AttributeNode::Attribute(Attribute {
        span: Span::new(start, end),
        name: name.to_string(),
        name_loc: Some(name_loc),
        value,
    }))
}

fn parse_attribute_value(parser_input: &mut ParserInput) -> ParseResult<AttributeValue> {
    let next = winnow::token::any.parse_next(parser_input)?;
    match next {
        '"' => {
            let val_start = parser_input.input.current_token_start();
            let content: &str =
                take_while(0.., |c: char| c != '"').parse_next(parser_input)?;
            let val_end = parser_input.input.previous_token_end();
            literal("\"").parse_next(parser_input)?;
            Ok(AttributeValue::Sequence(vec![
                AttributeSequenceValue::Text(Text {
                    span: Span::new(val_start, val_end),
                    data: content.to_string(),
                    raw: content.to_string(),
                }),
            ]))
        }
        '\'' => {
            let val_start = parser_input.input.current_token_start();
            let content: &str =
                take_while(0.., |c: char| c != '\'').parse_next(parser_input)?;
            let val_end = parser_input.input.previous_token_end();
            literal("'").parse_next(parser_input)?;
            Ok(AttributeValue::Sequence(vec![
                AttributeSequenceValue::Text(Text {
                    span: Span::new(val_start, val_end),
                    data: content.to_string(),
                    raw: content.to_string(),
                }),
            ]))
        }
        _ => {
            // Unquoted value: the first char is already consumed
            let rest: &str = take_while(0.., |c: char| {
                !c.is_ascii_whitespace() && !matches!(c, '>' | '/' | '=' | '{' | '}' | '<')
            })
            .parse_next(parser_input)?;
            let val_end = parser_input.input.previous_token_end();
            // Reconstruct full value: first char + rest
            let mut full = String::with_capacity(1 + rest.len());
            full.push(next);
            full.push_str(rest);
            let val_start = val_end - full.len();
            Ok(AttributeValue::Sequence(vec![
                AttributeSequenceValue::Text(Text {
                    span: Span::new(val_start, val_end),
                    data: full.clone(),
                    raw: full,
                }),
            ]))
        }
    }
}
