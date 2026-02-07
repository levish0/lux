use lux_ast::template::root::{Fragment, FragmentNode};
use winnow::Result;
use winnow::combinator::{dispatch, peek, repeat};
use winnow::prelude::*;
use winnow::token::any;

use crate::input::Input;
use crate::parser::state::element::parse_element;
use crate::parser::state::tag::parse_tag;
use crate::parser::state::text::parse_text;

/// Parse a top-level fragment (until EOF).
pub fn parse_fragment<'a>(input: &mut Input<'a>) -> Result<Fragment<'a>> {
    let nodes: Vec<FragmentNode<'a>> = repeat(
        0..,
        dispatch! {peek(any);
            '<' => parse_element,
            '{' => parse_tag,
            _ => parse_text.map(FragmentNode::Text),
        },
    )
    .parse_next(input)?;

    Ok(Fragment {
        nodes,
        transparent: false,
        dynamic: false,
    })
}

/// Parse a fragment inside an element, stopping when `</tag_name>` is encountered.
pub fn parse_fragment_until<'a>(input: &mut Input<'a>, closing_tag: &str) -> Result<Fragment<'a>> {
    parse_inner_fragment(input, Some(closing_tag))
}

/// Parse a fragment inside a block, stopping at `{:` or `{/`.
pub fn parse_block_fragment<'a>(input: &mut Input<'a>) -> Result<Fragment<'a>> {
    parse_inner_fragment(input, None)
}

fn parse_inner_fragment<'a>(
    input: &mut Input<'a>,
    closing_tag: Option<&str>,
) -> Result<Fragment<'a>> {
    let mut nodes = Vec::new();

    loop {
        let remaining: &str = &input.input;

        if remaining.is_empty() {
            break;
        }

        // Check for HTML closing tag
        if let Some(tag_name) = closing_tag {
            if let Some(after_slash) = remaining.strip_prefix("</") {
                let name_len = after_slash
                    .find(|c: char| {
                        !c.is_ascii_alphanumeric() && c != '-' && c != '_' && c != ':' && c != '.'
                    })
                    .unwrap_or(after_slash.len());
                if &after_slash[..name_len] == tag_name {
                    break;
                }
            }
        }

        // Check for block delimiter: {: or {/ (but not {/* or {//)
        if is_block_delimiter(remaining) {
            break;
        }

        let node = match remaining.as_bytes()[0] {
            b'<' => parse_element.parse_next(input)?,
            b'{' => parse_tag.parse_next(input)?,
            _ => parse_text.map(FragmentNode::Text).parse_next(input)?,
        };
        nodes.push(node);
    }

    Ok(Fragment {
        nodes,
        transparent: true,
        dynamic: false,
    })
}

fn is_block_delimiter(s: &str) -> bool {
    if let Some(rest) = s.strip_prefix('{') {
        let rest = rest.trim_start();
        rest.starts_with(':')
            || (rest.starts_with('/') && !rest.starts_with("/*") && !rest.starts_with("//"))
    } else {
        false
    }
}
