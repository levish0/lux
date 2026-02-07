use lux_ast::template::root::{Fragment, FragmentNode};
use winnow::Result;
use winnow::prelude::*;

use crate::input::Input;
use crate::parser::state::element::parse_element;
use crate::parser::state::tag::parse_tag;
use crate::parser::state::text::parse_text;

fn peek_tag_name(s: &str) -> Option<&str> {
    let after = s.strip_prefix('<')?;
    let end = after.find(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_' && c != ':' && c != '.')?;
    if end == 0 {
        return None;
    }
    Some(&after[..end])
}

/// Parse a top-level fragment (until EOF).
pub fn parse_fragment<'a>(input: &mut Input<'a>) -> Result<Fragment<'a>> {
    let mut nodes = Vec::new();

    loop {
        let remaining: &str = &input.input;
        if remaining.is_empty() {
            break;
        }

        match remaining.as_bytes()[0] {
            b'<' => {
                if let Some(node) = parse_element(input)? {
                    nodes.push(node);
                }
                // None = script/style consumed into ParserState
            }
            b'{' => {
                nodes.push(parse_tag.parse_next(input)?);
            }
            _ => {
                nodes.push(parse_text.map(FragmentNode::Text).parse_next(input)?);
            }
        }
    }

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
            if remaining.starts_with("</") {
                // Any closing tag ends this fragment.
                // If it matches our tag_name, element_body will consume it.
                // If it doesn't match, this element was auto-closed by an ancestor's closing tag.
                break;
            }

            // Check for opening tag that auto-closes the current element
            if remaining.starts_with('<') && !remaining.starts_with("<!")
                && let Some(next_name) = peek_tag_name(remaining)
                    && lux_utils::closing_tag::closing_tag_omitted(tag_name, Some(next_name)) {
                        break;
                    }
        }

        // Check for block delimiter: {: or {/ (but not {/* or {//)
        if is_block_delimiter(remaining) {
            break;
        }

        match remaining.as_bytes()[0] {
            b'<' => {
                if let Some(node) = parse_element(input)? {
                    nodes.push(node);
                }
            }
            b'{' => {
                nodes.push(parse_tag.parse_next(input)?);
            }
            _ => {
                nodes.push(parse_text.map(FragmentNode::Text).parse_next(input)?);
            }
        }
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
