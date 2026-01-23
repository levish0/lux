use svelte_ast::elements::{
    Component, RegularElement, SlotElement, SvelteBody, SvelteBoundary, SvelteDocument,
    SvelteFragment, SvelteHead, SvelteSelf, SvelteWindow, TitleElement,
};
use svelte_ast::node::{AttributeNode, FragmentNode};
use svelte_ast::root::Fragment;
use svelte_ast::span::Span;
use winnow::combinator::{opt, peek, repeat_till};
use winnow::prelude::*;
use winnow::stream::Location;
use winnow::token::{literal, take_while};
use winnow::Result as ParseResult;

use super::ParserInput;
use super::attribute::attribute_parser;
use super::fragment::fragment_node_parser;

pub fn element_parser(parser_input: &mut ParserInput) -> ParseResult<FragmentNode> {
    let start = parser_input.input.current_token_start();

    // consume '<'
    literal("<").parse_next(parser_input)?;

    let name_start = parser_input.input.current_token_start();
    let name = tag_name_parser(parser_input)?;
    let name_end = parser_input.input.previous_token_end();
    let name_loc = Span::new(name_start, name_end);

    // parse attributes
    let attributes = parse_attributes(parser_input)?;

    // Check if this element has shadowrootmode attribute
    let has_shadowrootmode = attributes.iter().any(|attr| {
        matches!(attr, AttributeNode::Attribute(a) if a.name == "shadowrootmode")
    });

    // check self-closing /> or >
    let self_closing = opt(literal("/")).parse_next(parser_input)?.is_some();
    literal(">").parse_next(parser_input)?;

    // Check shadowroot context BEFORE parsing children (for nested slot detection)
    let parent_is_shadowroot = parser_input.state.parent_is_shadowroot_template();

    let fragment = if self_closing || is_void_element(&name) {
        Fragment { nodes: vec![] }
    } else {
        // Push this element onto stack before parsing children
        parser_input.state.push_element(name.clone(), has_shadowrootmode);

        let (nodes, _): (Vec<FragmentNode>, _) = repeat_till(
            0..,
            fragment_node_parser,
            closing_tag_parser(&name),
        )
        .parse_next(parser_input)?;

        // Pop element from stack after parsing children
        parser_input.state.pop_element();

        Fragment { nodes }
    };

    let end = parser_input.input.previous_token_end();
    let span = Span::new(start, end);

    Ok(classify_element(&name, name_loc, attributes, fragment, span, parent_is_shadowroot || has_shadowrootmode))
}

fn parse_attributes(parser_input: &mut ParserInput) -> ParseResult<Vec<AttributeNode>> {
    let mut attributes = Vec::new();
    loop {
        // consume whitespace between attributes
        take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;
        // stop if we hit > or />
        if opt(peek(literal(">"))).parse_next(parser_input)?.is_some() {
            break;
        }
        if opt(peek(literal("/"))).parse_next(parser_input)?.is_some() {
            break;
        }
        attributes.push(attribute_parser(parser_input)?);
    }
    Ok(attributes)
}

fn tag_name_parser<'i>(parser_input: &mut ParserInput<'i>) -> ParseResult<String> {
    let name = take_while(1.., |c: char| {
        c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == ':'
    })
    .parse_next(parser_input)?;
    Ok(name.to_string())
}

fn closing_tag_parser<'a, 'i>(
    name: &'a str,
) -> impl FnMut(&mut ParserInput<'i>) -> ParseResult<()> + 'a {
    move |input: &mut ParserInput| {
        literal("</").parse_next(input)?;
        take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(input)?;
        literal(name).parse_next(input)?;
        take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(input)?;
        literal(">").parse_next(input)?;
        Ok(())
    }
}

fn classify_element(
    name: &str,
    name_loc: Span,
    attributes: Vec<AttributeNode>,
    fragment: Fragment,
    span: Span,
    is_inside_shadowroot: bool,
) -> FragmentNode {
    match name {
        "svelte:head" => FragmentNode::SvelteHead(SvelteHead {
            span,
            name_loc,
            fragment,
        }),
        "svelte:body" => FragmentNode::SvelteBody(SvelteBody {
            span,
            name_loc,
            attributes,
            fragment,
        }),
        "svelte:window" => FragmentNode::SvelteWindow(SvelteWindow {
            span,
            name_loc,
            attributes,
            fragment,
        }),
        "svelte:document" => FragmentNode::SvelteDocument(SvelteDocument {
            span,
            name_loc,
            attributes,
            fragment,
        }),
        "svelte:fragment" => FragmentNode::SvelteFragment(SvelteFragment {
            span,
            name_loc,
            attributes,
            fragment,
        }),
        "svelte:boundary" => FragmentNode::SvelteBoundary(SvelteBoundary {
            span,
            name_loc,
            attributes,
            fragment,
        }),
        "svelte:self" => FragmentNode::SvelteSelf(SvelteSelf {
            span,
            name_loc,
            attributes,
            fragment,
        }),
        "slot" => {
            // Inside <template shadowrootmode="...">, slot is a native HTML slot, not Svelte's
            if is_inside_shadowroot {
                FragmentNode::RegularElement(RegularElement {
                    span,
                    name: name.to_string(),
                    name_loc,
                    attributes,
                    fragment,
                })
            } else {
                FragmentNode::SlotElement(SlotElement {
                    span,
                    name: name.to_string(),
                    name_loc,
                    attributes,
                    fragment,
                })
            }
        }
        "title" => FragmentNode::TitleElement(TitleElement {
            span,
            name_loc,
            attributes,
            fragment,
        }),
        _ => {
            if name.starts_with(char::is_uppercase) {
                FragmentNode::Component(Component {
                    span,
                    name: name.to_string(),
                    name_loc,
                    attributes,
                    fragment,
                })
            } else {
                FragmentNode::RegularElement(RegularElement {
                    span,
                    name: name.to_string(),
                    name_loc,
                    attributes,
                    fragment,
                })
            }
        }
    }
}

fn is_void_element(name: &str) -> bool {
    matches!(
        name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}
