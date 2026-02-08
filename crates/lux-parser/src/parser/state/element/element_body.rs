use std::borrow::Cow;

use lux_ast::common::Span;
use lux_ast::template::attribute::{AttributeNode, AttributeValue};
use lux_ast::template::root::{Fragment, FragmentNode};
use lux_ast::template::tag::{ExpressionTag, Text, TextOrExpressionTag};
use oxc_ast::ast::Expression;
use winnow::Result;
use winnow::combinator::opt;
use winnow::error::ContextError;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::{any, literal, take_while};

use crate::input::Input;
use crate::parser::read::expression::read_expression;
use crate::parser::state::element::attribute::is_tag_name_char;
use crate::parser::state::fragment::parse_fragment_until;
use crate::parser::utils::helpers::skip_whitespace;

pub fn parse_element_body<'a>(input: &mut Input<'a>, name: &str) -> Result<(Fragment<'a>, usize)> {
    skip_whitespace(input);

    let self_closing = opt(literal("/>")).parse_next(input)?.is_some();
    if !self_closing {
        literal(">").parse_next(input)?;
    }

    let fragment = if self_closing || lux_utils::elements::is_void(name) {
        Fragment {
            nodes: Vec::new(),
            transparent: true,
            dynamic: false,
        }
    } else if name == "textarea" {
        let nodes = read_textarea_content(input)?;
        eat_closing_tag(input)?;
        Fragment {
            nodes,
            transparent: true,
            dynamic: false,
        }
    } else if name == "script" || name == "style" {
        let nodes = read_raw_text_content(input, name)?;
        eat_closing_tag(input)?;
        Fragment {
            nodes,
            transparent: true,
            dynamic: false,
        }
    } else {
        input.state.depth += 1;
        let f = parse_fragment_until(input, name)?;
        input.state.depth -= 1;

        // Graceful closing: consume </name> only if it matches
        let remaining: &str = &input.input;
        if let Some(stripped) = remaining.strip_prefix("</") {
            let after_slash = stripped.trim_start();
            let name_end = after_slash
                .find(|c: char| !is_tag_name_char(c))
                .unwrap_or(after_slash.len());
            if &after_slash[..name_end] == name {
                eat_closing_tag(input)?;
            }
            // else: ancestor's closing tag → don't consume (auto-closed)
        }
        // else: sibling opening tag caused auto-close → don't consume
        f
    };

    let end = input.previous_token_end();
    Ok((fragment, end))
}

pub fn extract_this_expression<'a>(
    attributes: &mut Vec<AttributeNode<'a>>,
) -> Result<Expression<'a>> {
    let this_idx = attributes.iter().position(|a| match a {
        AttributeNode::Attribute(attr) => attr.name == "this",
        _ => false,
    });

    if let Some(idx) = this_idx {
        let attr = attributes.remove(idx);
        match attr {
            AttributeNode::Attribute(a) => extract_expression_from_attr_value(a.value),
            _ => Err(ContextError::new()),
        }
    } else {
        Err(ContextError::new())
    }
}

fn extract_expression_from_attr_value(value: AttributeValue<'_>) -> Result<Expression<'_>> {
    match value {
        AttributeValue::ExpressionTag(et) => Ok(et.expression),
        AttributeValue::Sequence(mut seq) => {
            if seq.len() == 1 {
                match seq.remove(0) {
                    TextOrExpressionTag::ExpressionTag(et) => Ok(et.expression),
                    _ => Err(ContextError::new()),
                }
            } else {
                Err(ContextError::new())
            }
        }
        AttributeValue::True => Err(ContextError::new()),
    }
}

fn eat_closing_tag(input: &mut Input<'_>) -> Result<()> {
    literal("</").parse_next(input)?;
    skip_whitespace(input);
    let _: &str = take_while(1.., is_tag_name_char).parse_next(input)?;
    skip_whitespace(input);
    literal(">").parse_next(input)?;
    Ok(())
}

/// Read textarea content: text + expressions until `</textarea>` (case-insensitive).
fn read_textarea_content<'a>(input: &mut Input<'a>) -> Result<Vec<FragmentNode<'a>>> {
    let template = input.state.template;
    let allocator = input.state.allocator;
    let mut nodes: Vec<FragmentNode<'a>> = Vec::new();
    let mut text_start: Option<usize> = None;

    loop {
        let remaining: &str = &input.input;
        if remaining.is_empty() {
            break;
        }

        if remaining.len() >= 11
            && remaining.as_bytes()[0] == b'<'
            && remaining.as_bytes()[1] == b'/'
            && remaining[..11].eq_ignore_ascii_case("</textarea")
        {
            // Flush pending text
            if let Some(ts) = text_start.take() {
                let end = input.current_token_start();
                let raw = &template[ts..end];
                let decoded = lux_utils::html_entities::decode_character_references(raw, false);
                let data = match decoded {
                    Cow::Borrowed(_) => raw,
                    Cow::Owned(s) => allocator.alloc_str(&s),
                };
                nodes.push(FragmentNode::Text(Text {
                    span: Span::new(ts as u32, end as u32),
                    data,
                    raw,
                }));
            }
            break;
        }

        if remaining.starts_with('{') {
            // Flush pending text
            if let Some(ts) = text_start.take() {
                let end = input.current_token_start();
                let raw = &template[ts..end];
                let decoded = lux_utils::html_entities::decode_character_references(raw, false);
                let data = match decoded {
                    Cow::Borrowed(_) => raw,
                    Cow::Owned(s) => allocator.alloc_str(&s),
                };
                nodes.push(FragmentNode::Text(Text {
                    span: Span::new(ts as u32, end as u32),
                    data,
                    raw,
                }));
            }

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

/// Read raw text content for nested `<script>` / `<style>` (no expression parsing).
fn read_raw_text_content<'a>(input: &mut Input<'a>, name: &str) -> Result<Vec<FragmentNode<'a>>> {
    let template = input.state.template;
    let start = input.current_token_start();
    let close_tag = format!("</{}>", name);

    loop {
        let remaining: &str = &input.input;
        if remaining.is_empty() {
            break;
        }
        if remaining.starts_with(close_tag.as_str()) {
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
