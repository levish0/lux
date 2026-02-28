use lux_ast::common::Span;
use lux_ast::template::attribute::AttributeValue;
use lux_ast::template::tag::{Text, TextOrExpressionTag};
use winnow::Result;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::take_while;

use crate::input::Input;

pub(super) fn read_static_value<'a>(input: &mut Input<'a>) -> Result<AttributeValue<'a>> {
    let remaining: &str = &input.input;

    let (quote, content) = if remaining.starts_with('"') {
        input.next_slice(1);
        let val: &str = take_while(0.., |c: char| c != '"').parse_next(input)?;
        input.next_slice(1); // closing "
        ('"', val)
    } else if remaining.starts_with('\'') {
        input.next_slice(1);
        let val: &str = take_while(0.., |c: char| c != '\'').parse_next(input)?;
        input.next_slice(1); // closing '
        ('\'', val)
    } else {
        let val: &str = take_while(1.., |c: char| {
            !c.is_ascii_whitespace() && c != '>' && c != '/'
        })
        .parse_next(input)?;
        (' ', val)
    };

    let start = input.current_token_start() - content.len() - if quote != ' ' { 1 } else { 0 };
    let end = input.previous_token_end();

    Ok(AttributeValue::Sequence(vec![TextOrExpressionTag::Text(
        Text {
            span: Span::new(start as u32, end as u32),
            data: content,
            raw: content,
        },
    )]))
}
