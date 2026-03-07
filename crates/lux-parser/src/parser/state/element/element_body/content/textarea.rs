use lux_ast::common::Span;
use lux_ast::template::root::FragmentNode;
use lux_ast::template::tag::ExpressionTag;
use winnow::Result;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::{any, literal};

use crate::input::Input;
use crate::parser::read::expression::read_expression_until;
use crate::parser::utils::helpers::skip_whitespace;

use super::text_node::flush_text_node;

pub(in crate::parser::state::element::element_body) fn read_textarea_content<'a>(
    input: &mut Input<'a>,
) -> Result<Vec<FragmentNode<'a>>> {
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
            let expression = read_expression_until(input, b"")?;
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

fn starts_with_case_insensitive_textarea_close(remaining: &str) -> bool {
    remaining.len() >= 10
        && remaining.as_bytes()[0] == b'<'
        && remaining.as_bytes()[1] == b'/'
        && remaining[..10].eq_ignore_ascii_case("</textarea")
}
