use std::borrow::Cow;

use lux_ast::common::Span;
use lux_ast::template::root::FragmentNode;
use lux_ast::template::tag::{ExpressionTag, Text};
use winnow::Result;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::{any, literal};

use crate::input::Input;
use crate::parser::read::expression::read_expression;
use crate::parser::utils::helpers::skip_whitespace;

pub(super) fn read_textarea_content<'a>(input: &mut Input<'a>) -> Result<Vec<FragmentNode<'a>>> {
    let template = input.state.template;
    let allocator = input.state.allocator;
    let mut nodes: Vec<FragmentNode<'a>> = Vec::new();
    let mut text_start: Option<usize> = None;

    loop {
        let remaining: &str = &input.input;
        if remaining.is_empty() {
            break;
        }

        if starts_with_case_insensitive_textarea_close(remaining) {
            flush_text_node(input, &mut nodes, &mut text_start, template, allocator);
            break;
        }

        if remaining.starts_with('{') {
            flush_text_node(input, &mut nodes, &mut text_start, template, allocator);

            let expr_start = input.current_token_start();
            literal("{").parse_next(input)?;
            skip_whitespace(input);
            let expression = read_expression(input)?;
            skip_whitespace(input);
            literal("}").parse_next(input)?;
            let expr_end = input.previous_token_end();

            nodes.push(FragmentNode::ExpressionTag(ExpressionTag {
                span: Span::new(expr_start as u32, expr_end as u32),
                expression,
            }));
        } else {
            if text_start.is_none() {
                text_start = Some(input.current_token_start());
            }
            let _: char = any.parse_next(input)?;
        }
    }

    Ok(nodes)
}

pub(super) fn read_raw_text_content<'a>(
    input: &mut Input<'a>,
    name: &str,
) -> Result<Vec<FragmentNode<'a>>> {
    let template = input.state.template;
    let start = input.current_token_start();
    let close_tag = format!("</{}>", name);

    loop {
        let remaining: &str = &input.input;
        if remaining.is_empty() || remaining.starts_with(close_tag.as_str()) {
            break;
        }
        let _: char = any.parse_next(input)?;
    }

    let end = input.current_token_start();
    let data = &template[start..end];

    if data.is_empty() {
        return Ok(Vec::new());
    }

    Ok(vec![FragmentNode::Text(Text {
        span: Span::new(start as u32, end as u32),
        data,
        raw: data,
    })])
}

fn starts_with_case_insensitive_textarea_close(remaining: &str) -> bool {
    remaining.len() >= 11
        && remaining.as_bytes()[0] == b'<'
        && remaining.as_bytes()[1] == b'/'
        && remaining[..11].eq_ignore_ascii_case("</textarea")
}

fn flush_text_node<'a>(
    input: &mut Input<'a>,
    nodes: &mut Vec<FragmentNode<'a>>,
    text_start: &mut Option<usize>,
    template: &'a str,
    allocator: &'a oxc_allocator::Allocator,
) {
    let Some(start) = text_start.take() else {
        return;
    };

    let end = input.current_token_start();
    let raw = &template[start..end];
    let decoded = lux_utils::html_entities::decode_character_references(raw, false);
    let data = match decoded {
        Cow::Borrowed(_) => raw,
        Cow::Owned(s) => allocator.alloc_str(&s),
    };

    nodes.push(FragmentNode::Text(Text {
        span: Span::new(start as u32, end as u32),
        data,
        raw,
    }));
}
