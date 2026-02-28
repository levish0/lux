mod const_tag;
mod debug_tag;
mod html_tag;
mod render_tag;

use lux_ast::template::root::FragmentNode;
use winnow::Result;
use winnow::error::ContextError;

use crate::input::Input;

pub fn parse_special<'a>(input: &mut Input<'a>, start: usize) -> Result<FragmentNode<'a>> {
    let remaining: &str = &input.input;
    if remaining.starts_with("@html") {
        html_tag::parse_html_tag(input, start)
    } else if remaining.starts_with("@const") {
        const_tag::parse_const_tag(input, start)
    } else if remaining.starts_with("@debug") {
        debug_tag::parse_debug_tag(input, start)
    } else if remaining.starts_with("@render") {
        render_tag::parse_render_tag(input, start)
    } else {
        Err(ContextError::new())
    }
}
