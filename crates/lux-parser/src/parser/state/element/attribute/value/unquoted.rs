use lux_ast::common::Span;
use lux_ast::template::attribute::AttributeValue;
use lux_ast::template::tag::TextOrExpressionTag;
use winnow::Result;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::take_while;

use crate::input::Input;

use super::decode::decode_attr_text;

pub(super) fn parse_unquoted_value<'a>(input: &mut Input<'a>) -> Result<AttributeValue<'a>> {
    let value: &str = take_while(1.., |c: char| {
        !c.is_ascii_whitespace()
            && c != '>'
            && c != '/'
            && c != '"'
            && c != '\''
            && c != '='
            && c != '`'
    })
    .parse_next(input)?;

    let end = input.previous_token_end();
    let start = end - value.len();
    let span = Span::new(start as u32, end as u32);
    let text = decode_attr_text(value, span, input.state.allocator);

    Ok(AttributeValue::Sequence(vec![TextOrExpressionTag::Text(
        text,
    )]))
}
