use lux_ast::common::Span;
use lux_ast::template::root::FragmentNode;
use lux_ast::template::tag::Comment;
use winnow::Result;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::{literal, take_until};

use crate::input::Input;

/// Parse an HTML comment: `<!-- ... -->`.
/// Assumes `<` has already been consumed; starts at `!--`.
pub fn parse_comment<'a>(input: &mut Input<'a>, start: usize) -> Result<FragmentNode<'a>> {
    literal("!--").parse_next(input)?;

    let data: &str = take_until(0.., "-->").parse_next(input)?;
    literal("-->").parse_next(input)?;

    let end = input.previous_token_end();

    Ok(FragmentNode::Comment(Comment {
        span: Span::new(start as u32, end as u32),
        data,
    }))
}
