use svelte_ast::elements::{
    Component, RegularElement, SlotElement, SvelteBody, SvelteBoundary, SvelteDocument,
    SvelteFragment, SvelteHead, SvelteSelf, SvelteWindow, TitleElement,
};
use svelte_ast::node::{AttributeNode, FragmentNode};
use svelte_ast::root::Fragment;
use svelte_ast::span::Span;
use winnow::combinator::{peek, repeat_till};
use winnow::prelude::*;
use winnow::stream::Location;
use winnow::token::{any, literal, take_while};
use winnow::Result;

use super::ParserInput;
use super::attribute::attribute_parser;
use super::fragment::fragment_node_parser;

pub fn element_parser(parser_input: &mut ParserInput) -> Result<FragmentNode> {
    let start = parser_input.input.current_token_start();

    // consume '<'
    literal("<").parse_next(parser_input)?;

    let name_start = parser_input.input.current_token_start();
    let name = tag_name_parser(parser_input)?;
    let name_end = parser_input.input.previous_token_end();
    let name_loc = Span::new(name_start, name_end);

    // parse attributes
    let attributes = parse_attributes(parser_input)?;

    // check self-closing /> or >
    let self_closing = peek(any).parse_next(parser_input)? == '/';
    if self_closing {
        literal("/").parse_next(parser_input)?;
    }
    literal(">").parse_next(parser_input)?;

    let fragment = if self_closing || is_void_element(&name) {
        Fragment { nodes: vec![] }
    } else {
        let (nodes, _): (Vec<FragmentNode>, _) = repeat_till(
            0..,
            fragment_node_parser,
            closing_tag_parser(&name),
        )
        .parse_next(parser_input)?;
        Fragment { nodes }
    };

    let end = parser_input.input.previous_token_end();
    let span = Span::new(start, end);

    Ok(classify_element(&name, name_loc, attributes, fragment, span))
}

fn parse_attributes(parser_input: &mut ParserInput) -> Result<Vec<AttributeNode>> {
    let mut attributes = Vec::new();
    loop {
        // consume whitespace between attributes
        take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;
        // stop if we hit > or />
        let next = peek(any).parse_next(parser_input)?;
        if next == '>' || next == '/' {
            break;
        }
        attributes.push(attribute_parser(parser_input)?);
    }
    Ok(attributes)
}

fn tag_name_parser<'i>(parser_input: &mut ParserInput<'i>) -> Result<String> {
    let name = take_while(1.., |c: char| {
        c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == ':'
    })
    .parse_next(parser_input)?;
    Ok(name.to_string())
}

fn closing_tag_parser<'a, 'i>(
    name: &'a str,
) -> impl FnMut(&mut ParserInput<'i>) -> Result<()> + 'a {
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
        "slot" => FragmentNode::SlotElement(SlotElement {
            span,
            name: name.to_string(),
            name_loc,
            attributes,
            fragment,
        }),
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
