use lux_ast::common::Span;
use lux_ast::template::tag::Text;
use winnow::Result;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::take_while;

use crate::input::Input;

pub fn parse_text<'a>(input: &mut Input<'a>) -> Result<Text<'a>> {
    let start = input.current_token_start();

    let raw = take_while(1.., |c: char| c != '<' && c != '{').parse_next(input)?;

    let end = input.previous_token_end();

    Ok(Text {
        span: Span::new(start as u32, end as u32),
        data: raw,
        raw,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxc_allocator::Allocator;
    use winnow::stream::{LocatingSlice, Stateful};

    use crate::input::ParserState;

    fn make_input<'a>(allocator: &'a Allocator, template: &'a str) -> Input<'a> {
        Stateful {
            input: LocatingSlice::new(template),
            state: ParserState::new(allocator, template, false),
        }
    }

    #[test]
    fn test_parse_simple_text() {
        let allocator = Allocator::default();
        let mut input = make_input(&allocator, "hello world");
        let text = parse_text(&mut input).unwrap();
        assert_eq!(text.data, "hello world");
        assert_eq!(text.span, Span::new(0, 11));
    }

    #[test]
    fn test_parse_text_stops_at_tag() {
        let allocator = Allocator::default();
        let mut input = make_input(&allocator, "hello <div>");
        let text = parse_text(&mut input).unwrap();
        assert_eq!(text.data, "hello ");
    }

    #[test]
    fn test_parse_text_stops_at_brace() {
        let allocator = Allocator::default();
        let mut input = make_input(&allocator, "hello {name}");
        let text = parse_text(&mut input).unwrap();
        assert_eq!(text.data, "hello ");
    }
}
