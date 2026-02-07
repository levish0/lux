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
pub fn parse_fragment_until<'a>(
    input: &mut Input<'a>,
    closing_tag: &str,
) -> Result<Fragment<'a>> {
    let mut nodes = Vec::new();

    loop {
        let remaining: &str = &input.input;

        // Check for EOF
        if remaining.is_empty() {
            break;
        }

        // Check for closing tag
        if let Some(after_slash) = remaining.strip_prefix("</") {
            // Peek at the tag name
            let name_len = after_slash
                .find(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_' && c != ':')
                .unwrap_or(after_slash.len());
            let peeked_name = &after_slash[..name_len];

            if peeked_name == closing_tag {
                break;
            }
        }

        // Check for block continuation/closing `{:` or `{/`
        if remaining.starts_with("{:") || remaining.starts_with("{/") {
            break;
        }

        // Parse next node
        let next = remaining.as_bytes()[0];
        let node = match next {
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
