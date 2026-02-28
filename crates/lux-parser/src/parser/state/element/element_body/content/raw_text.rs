use lux_ast::common::Span;
use lux_ast::template::root::FragmentNode;
use lux_ast::template::tag::Text;
use winnow::Result;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::any;

use crate::input::Input;

pub(in crate::parser::state::element::element_body) fn read_raw_text_content<'a>(
    input: &mut Input<'a>,
    name: &str,
) -> Result<Vec<FragmentNode<'a>>> {
    let template = input.state.template;
    let start = input.current_token_start();
    let close_tag = format!("</{}>", name);

    loop {
        let remaining: &str = &input.input;
        if remaining.is_empty() || remaining.starts_with(close_tag.as_str()) {
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
