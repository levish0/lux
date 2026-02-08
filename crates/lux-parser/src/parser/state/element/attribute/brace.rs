use lux_ast::common::Span;
use lux_ast::template::attribute::{Attribute, AttributeNode, AttributeValue, SpreadAttribute};
use lux_ast::template::tag::{AttachTag, ExpressionTag};
use oxc_ast::ast::Expression;
use winnow::Result;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::literal;

use crate::input::Input;
use crate::parser::read::expression::read_expression;
use crate::parser::utils::helpers::{require_whitespace, skip_whitespace};

pub fn parse_brace_attribute<'a>(input: &mut Input<'a>) -> Result<AttributeNode<'a>> {
    let start = input.current_token_start();
    literal("{").parse_next(input)?;
    skip_whitespace(input);

    let remaining: &str = &input.input;

    if remaining.starts_with("...") {
        return parse_spread(input, start);
    }

    if remaining.starts_with("@attach") {
        return parse_attach(input, start);
    }

    parse_shorthand(input, start)
}

fn parse_spread<'a>(input: &mut Input<'a>, start: usize) -> Result<AttributeNode<'a>> {
    literal("...").parse_next(input)?;
    let expression = read_expression(input)?;
    skip_whitespace(input);
    literal("}").parse_next(input)?;
    let end = input.previous_token_end();

    Ok(AttributeNode::SpreadAttribute(SpreadAttribute {
        span: Span::new(start as u32, end as u32),
        expression,
    }))
}

fn parse_attach<'a>(input: &mut Input<'a>, start: usize) -> Result<AttributeNode<'a>> {
    literal("@attach").parse_next(input)?;
    require_whitespace(input)?;
    let expression = read_expression(input)?;
    skip_whitespace(input);
    literal("}").parse_next(input)?;
    let end = input.previous_token_end();

    Ok(AttributeNode::AttachTag(AttachTag {
        span: Span::new(start as u32, end as u32),
        expression,
    }))
}

fn parse_shorthand<'a>(input: &mut Input<'a>, start: usize) -> Result<AttributeNode<'a>> {
    let expression = read_expression(input)?;
    let name = get_expression_name(&expression);
    skip_whitespace(input);
    literal("}").parse_next(input)?;
    let end = input.previous_token_end();

    let value = AttributeValue::ExpressionTag(ExpressionTag {
        span: Span::new(start as u32, end as u32),
        expression,
    });

    Ok(AttributeNode::Attribute(Attribute {
        span: Span::new(start as u32, end as u32),
        name,
        value,
    }))
}

fn get_expression_name<'a>(expr: &Expression<'a>) -> &'a str {
    match expr {
        Expression::Identifier(id) => id.name.as_str(),
        _ => "",
    }
}
