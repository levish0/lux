mod stop;

use lux_ast::template::root::{Fragment, FragmentNode};
use winnow::Result;
use winnow::prelude::*;

use self::stop::{is_block_delimiter, should_stop_for_element_fragment};
use crate::context::with_depth;
use crate::input::Input;
use crate::parser::state::element::parse_element;
use crate::parser::state::tag::parse_tag;
use crate::parser::state::text::parse_text;

#[derive(Clone, Copy)]
enum FragmentBoundary<'a> {
    TopLevel,
    Element(&'a str),
    Block,
}

/// Parse a top-level fragment (until EOF).
pub fn parse_fragment<'a>(input: &mut Input<'a>) -> Result<Fragment<'a>> {
    parse_nodes_until(input, FragmentBoundary::TopLevel, false)
}

/// Parse a fragment inside an element, stopping when a closing-tag
/// or block delimiter is encountered.
pub fn parse_fragment_until<'a>(input: &mut Input<'a>, closing_tag: &str) -> Result<Fragment<'a>> {
    with_depth(input, |inner| {
        parse_nodes_until(inner, FragmentBoundary::Element(closing_tag), true)
    })
}

/// Parse a fragment inside a block, stopping at `{:...}` or `{/...}`.
pub fn parse_block_fragment<'a>(input: &mut Input<'a>) -> Result<Fragment<'a>> {
    with_depth(input, |inner| {
        parse_nodes_until(inner, FragmentBoundary::Block, true)
    })
}

fn parse_nodes_until<'a>(
    input: &mut Input<'a>,
    boundary: FragmentBoundary<'_>,
    transparent: bool,
) -> Result<Fragment<'a>> {
    let mut nodes = Vec::new();

    loop {
        let remaining: &str = &input.input;
        if remaining.is_empty() || should_stop(boundary, remaining) {
            break;
        }

        match remaining.as_bytes()[0] {
            b'<' => {
                if let Some(node) = parse_element(input)? {
                    nodes.push(node);
                }
            }
            b'{' => nodes.push(parse_tag.parse_next(input)?),
            _ => nodes.push(parse_text.map(FragmentNode::Text).parse_next(input)?),
        }
    }

    Ok(Fragment {
        nodes,
        transparent,
        dynamic: false,
    })
}

fn should_stop(boundary: FragmentBoundary<'_>, source: &str) -> bool {
    match boundary {
        FragmentBoundary::TopLevel => false,
        FragmentBoundary::Element(tag_name) => {
            is_block_delimiter(source) || should_stop_for_element_fragment(source, tag_name)
        }
        FragmentBoundary::Block => is_block_delimiter(source),
    }
}
