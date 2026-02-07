use lux_ast::template::attribute::{AttributeNode, AttributeValue};
use lux_ast::template::root::Fragment;
use lux_ast::template::tag::TextOrExpressionTag;
use oxc_ast::ast::Expression;
use winnow::Result;
use winnow::combinator::opt;
use winnow::error::ContextError;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::{literal, take_while};

use crate::input::Input;
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
    } else {
        input.state.depth += 1;
        let f = parse_fragment_until(input, name)?;
        input.state.depth -= 1;

        // Graceful closing: consume </name> only if it matches
        let remaining: &str = &input.input;
        if remaining.starts_with("</") {
            let after_slash = &remaining[2..].trim_start();
            let name_end = after_slash
                .find(|c: char| !is_tag_name_char(c))
                .unwrap_or(after_slash.len());
            if &after_slash[..name_end] == name {
                // Normal close: consume </name>
                literal("</").parse_next(input)?;
                skip_whitespace(input);
                let _: &str = take_while(1.., is_tag_name_char).parse_next(input)?;
                skip_whitespace(input);
                literal(">").parse_next(input)?;
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
