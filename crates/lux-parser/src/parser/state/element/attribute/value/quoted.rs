use lux_ast::template::attribute::AttributeValue;
use lux_ast::template::tag::TextOrExpressionTag;
use winnow::Result;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::any;

use crate::input::Input;

use super::sequence::read_sequence;

pub(super) fn parse_quoted_value<'a>(
    input: &mut Input<'a>,
    quote: u8,
) -> Result<AttributeValue<'a>> {
    let _: char = any.parse_next(input)?;

    let chunks = read_sequence(input, quote)?;

    // Consume closing quote
    let _: char = any.parse_next(input)?;

    // Single expression without surrounding text -> ExpressionTag
    if chunks.len() == 1 && matches!(chunks.first(), Some(TextOrExpressionTag::ExpressionTag(_))) {
        let chunk = chunks.into_iter().next().expect("single item");
        if let TextOrExpressionTag::ExpressionTag(expression_tag) = chunk {
            return Ok(AttributeValue::ExpressionTag(expression_tag));
        }
        unreachable!();
    }

    Ok(AttributeValue::Sequence(chunks))
}
