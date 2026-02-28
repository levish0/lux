use std::borrow::Cow;

use lux_ast::common::Span;
use lux_ast::template::root::FragmentNode;
use lux_ast::template::tag::Text;
use winnow::stream::Location as StreamLocation;

use crate::input::Input;

pub(super) fn flush_text_node<'a>(
    input: &mut Input<'a>,
    nodes: &mut Vec<FragmentNode<'a>>,
    text_start: &mut Option<usize>,
    template: &'a str,
    allocator: &'a oxc_allocator::Allocator,
) {
    let Some(start) = text_start.take() else {
        return;
    };

    let end = input.current_token_start();
    let raw = &template[start..end];
    let decoded = lux_utils::html_entities::decode_character_references(raw, false);
    let data = match decoded {
        Cow::Borrowed(_) => raw,
        Cow::Owned(s) => allocator.alloc_str(&s),
    };

    nodes.push(FragmentNode::Text(Text {
        span: Span::new(start as u32, end as u32),
        data,
        raw,
    }));
}
