use lux_ast::template::attribute::AttributeValue;
use lux_ast::template::tag::TextOrExpressionTag;
use oxc_ast::ast::Expression;
use winnow::Result;
use winnow::error::ContextError;

use crate::input::Input;

pub(super) fn extract_directive_expression<'a>(
    input: &mut Input<'a>,
    value: Option<AttributeValue<'a>>,
    name: &'a str,
) -> Result<Expression<'a>> {
    match value {
        Some(value) => extract_expression_from_value(value),
        None => parse_identifier_expression(input, name),
    }
}

pub(super) fn extract_expression_from_value(value: AttributeValue<'_>) -> Result<Expression<'_>> {
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

fn parse_identifier_expression<'a>(input: &mut Input<'a>, name: &'a str) -> Result<Expression<'a>> {
    let allocator = input.state.allocator;
    let source_type = if input.state.ts {
        oxc_span::SourceType::ts()
    } else {
        oxc_span::SourceType::mjs()
    };

    oxc_parser::Parser::new(allocator, name, source_type)
        .parse_expression()
        .map_err(|_| ContextError::new())
}
