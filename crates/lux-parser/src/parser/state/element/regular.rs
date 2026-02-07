use lux_ast::common::Span;
use lux_ast::template::attribute::{Attribute, AttributeNode, AttributeValue};
use lux_ast::template::element::RegularElement;
use lux_ast::template::root::{Fragment, FragmentNode};
use lux_ast::template::tag::{Text, TextOrExpressionTag};
use winnow::Result;
use winnow::combinator::opt;
use winnow::error::ContextError;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::{any, literal, take_while};

use crate::input::Input;
use crate::parser::state::fragment::parse_fragment_until;
use crate::parser::utils::helpers::{peek_str, skip_whitespace};

/// Parse an opening tag and its children.
/// Assumes `<` has already been consumed.
pub fn parse_opening_tag<'a>(input: &mut Input<'a>, start: usize) -> Result<FragmentNode<'a>> {
    let name: &str = take_while(1.., is_tag_name_char).parse_next(input)?;

    let attributes = parse_attributes(input)?;

    skip_whitespace(input);

    // Check for self-closing />
    let self_closing = opt(literal("/>")).parse_next(input)?.is_some();

    if !self_closing {
        literal(">").parse_next(input)?;
    }

    if self_closing || is_void_element(name) {
        let end = input.previous_token_end();
        return Ok(FragmentNode::RegularElement(RegularElement {
            span: Span::new(start as u32, end as u32),
            name,
            attributes,
            fragment: Fragment {
                nodes: Vec::new(),
                transparent: true,
                dynamic: false,
            },
            metadata: None,
        }));
    }

    let fragment = parse_fragment_until(input, name)?;

    literal("</").parse_next(input)?;
    skip_whitespace(input);
    let close_name: &str = take_while(1.., is_tag_name_char).parse_next(input)?;
    if close_name != name {
        return Err(ContextError::new());
    }
    skip_whitespace(input);
    literal(">").parse_next(input)?;

    let end = input.previous_token_end();

    Ok(FragmentNode::RegularElement(RegularElement {
        span: Span::new(start as u32, end as u32),
        name,
        attributes,
        fragment,
        metadata: None,
    }))
}

fn parse_attributes<'a>(input: &mut Input<'a>) -> Result<Vec<AttributeNode<'a>>> {
    let mut attrs = Vec::new();

    loop {
        skip_whitespace(input);

        if peek_str(input, ">") || peek_str(input, "/>") {
            break;
        }

        let remaining: &str = &(*input.input);
        if remaining.is_empty() {
            break;
        }

        let attr_start = input.current_token_start();
        let name_result: core::result::Result<&str, ContextError> =
            take_while(1.., is_attr_name_char).parse_next(input);

        let name = match name_result {
            Ok(n) => n,
            Err(_) => break,
        };

        skip_whitespace(input);

        let value = if opt(literal("=")).parse_next(input)?.is_some() {
            skip_whitespace(input);
            parse_attribute_value(input)?
        } else {
            AttributeValue::True
        };

        let attr_end = input.previous_token_end();

        attrs.push(AttributeNode::Attribute(Attribute {
            span: Span::new(attr_start as u32, attr_end as u32),
            name,
            value,
            metadata: None,
        }));
    }

    Ok(attrs)
}

fn parse_attribute_value<'a>(input: &mut Input<'a>) -> Result<AttributeValue<'a>> {
    let remaining: &str = &(*input.input);
    let first = remaining.as_bytes().first().copied();

    match first {
        Some(b'"') | Some(b'\'') => {
            let quote = first.unwrap() as char;
            // Consume opening quote
            let _: char = any.parse_next(input)?;

            let value: &str =
                take_while(0.., move |c: char| c != quote).parse_next(input)?;

            // Consume closing quote
            let _: char = any.parse_next(input)?;

            if value.is_empty() {
                Ok(AttributeValue::Sequence(Vec::new()))
            } else {
                let end = input.previous_token_end();
                let start = end - value.len() - 1;
                let text = Text {
                    span: Span::new(start as u32, (start + value.len()) as u32),
                    data: value,
                    raw: value,
                };
                Ok(AttributeValue::Sequence(vec![
                    TextOrExpressionTag::Text(text),
                ]))
            }
        }
        _ => {
            let value: &str = take_while(1.., |c: char| {
                !c.is_ascii_whitespace() && c != '>' && c != '/'
            })
            .parse_next(input)?;

            let end = input.previous_token_end();
            let start = end - value.len();
            let text = Text {
                span: Span::new(start as u32, end as u32),
                data: value,
                raw: value,
            };
            Ok(AttributeValue::Sequence(vec![
                TextOrExpressionTag::Text(text),
            ]))
        }
    }
}

fn is_tag_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ':' || c == '.'
}

fn is_attr_name_char(c: char) -> bool {
    !c.is_ascii_whitespace() && c != '=' && c != '>' && c != '/' && c != '"' && c != '\''
}

fn is_void_element(name: &str) -> bool {
    lux_utils::elements::is_void(name)
}
