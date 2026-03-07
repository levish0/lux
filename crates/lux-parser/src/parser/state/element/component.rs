use lux_ast::common::Span;
use lux_ast::template::element::Component;
use lux_ast::template::root::{Fragment, FragmentNode};
use winnow::Result;
use winnow::combinator::opt;
use winnow::error::ContextError;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::{literal, take_while};

use crate::input::Input;
use crate::parser::state::element::attribute::{is_tag_name_char, parse_attributes};
use crate::parser::state::fragment::parse_fragment_until;
use crate::parser::utils::helpers::skip_whitespace;

pub fn parse_component<'a>(
    input: &mut Input<'a>,
    start: usize,
    name: &'a str,
) -> Result<FragmentNode<'a>> {
    let attributes = parse_attributes(input)?;
    skip_whitespace(input);

    let self_closing = opt(literal("/>")).parse_next(input)?.is_some();
    if !self_closing {
        let has_open_close = opt(literal(">")).parse_next(input)?.is_some();
        if !has_open_close {
            if input.state.loose {
                return Ok(FragmentNode::Component(Component {
                    span: Span::new(start as u32, input.current_token_start() as u32),
                    name,
                    attributes,
                    fragment: Fragment {
                        nodes: Vec::new(),
                        transparent: true,
                        dynamic: false,
                    },
                }));
            }
            return Err(ContextError::new());
        }
    }

    let fragment = if self_closing {
        Fragment {
            nodes: Vec::new(),
            transparent: true,
            dynamic: false,
        }
    } else {
        let f = parse_fragment_until(input, name)?;
        let remaining: &str = &input.input;
        if peek_closing_tag_name(remaining) == Some(name) {
            literal("</").parse_next(input)?;
            skip_whitespace(input);
            let _: &str = take_while(1.., is_tag_name_char).parse_next(input)?;
            skip_whitespace(input);
            literal(">").parse_next(input)?;
            f
        } else if input.state.loose {
            f
        } else {
            return Err(ContextError::new());
        }
    };

    let end = input.previous_token_end();

    Ok(FragmentNode::Component(Component {
        span: Span::new(start as u32, end as u32),
        name,
        attributes,
        fragment,
    }))
}

fn peek_closing_tag_name(source: &str) -> Option<&str> {
    let stripped = source.strip_prefix("</")?;
    let trimmed = stripped.trim_start();
    let name_end = trimmed
        .find(|c: char| !is_tag_name_char(c))
        .unwrap_or(trimmed.len());

    if name_end == 0 {
        return None;
    }

    Some(&trimmed[..name_end])
}
