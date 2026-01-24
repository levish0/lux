use svelte_ast::JsNode;
use svelte_ast::attributes::{AttributeSequenceValue, AttributeValue};
use svelte_ast::elements::{
    Component, RegularElement, SlotElement, SvelteBody, SvelteBoundary, SvelteComponent,
    SvelteDocument, SvelteElement, SvelteFragment, SvelteHead, SvelteOptionsRaw, SvelteSelf,
    SvelteWindow, TitleElement,
};
use svelte_ast::node::{AttributeNode, FragmentNode};
use svelte_ast::root::Fragment;
use svelte_ast::span::{SourceLocation, Span};
use winnow::Result as ParseResult;
use winnow::combinator::{opt, peek, repeat_till};
use winnow::prelude::*;
use winnow::stream::Location;
use winnow::token::{literal, take_while};

use super::ParserInput;
use super::attribute::attribute_parser;
use super::fragment::fragment_node_parser;
use crate::error::{ErrorKind, ParseError};

const ROOT_ONLY_META_TAGS: &[&str] = &[
    "svelte:head",
    "svelte:options",
    "svelte:window",
    "svelte:document",
    "svelte:body",
];

const ALL_META_TAGS: &[&str] = &[
    "svelte:head",
    "svelte:options",
    "svelte:window",
    "svelte:document",
    "svelte:body",
    "svelte:element",
    "svelte:component",
    "svelte:self",
    "svelte:fragment",
    "svelte:boundary",
];

pub fn element_parser(parser_input: &mut ParserInput) -> ParseResult<FragmentNode> {
    let start = parser_input.input.current_token_start();
    let loose = parser_input.state.loose;

    // consume '<'
    literal("<").parse_next(parser_input)?;

    let name_start = parser_input.input.current_token_start();
    let name = if loose {
        tag_name_parser_loose(parser_input)?
    } else {
        tag_name_parser(parser_input)?
    };
    let name_end = parser_input.input.previous_token_end();
    let name_loc = parser_input.state.locator.locate_span(name_start, name_end);

    // Validate svelte: meta tags (skip in loose mode)
    if !loose && name.starts_with("svelte:") {
        if !ALL_META_TAGS.contains(&name.as_str()) {
            let list = ALL_META_TAGS
                .iter()
                .map(|t| format!("`<{}>`", t))
                .collect::<Vec<_>>()
                .join(", ");
            parser_input.state.errors.push(ParseError::new(
                ErrorKind::SvelteMetaInvalidTag,
                Span::new(name_start, name_end),
                format!("Valid `<svelte:...>` tag names are {}", list),
            ));
            return Err(winnow::error::ContextError::new());
        }
        if ROOT_ONLY_META_TAGS.contains(&name.as_str()) {
            // Must be at root level (no parent elements)
            if !parser_input.state.element_stack.is_empty() {
                parser_input.state.errors.push(ParseError::new(
                    ErrorKind::SvelteMetaInvalidPlacement,
                    Span::new(name_start, name_end),
                    format!("`<{}>` tags cannot be inside elements or blocks", name),
                ));
                return Err(winnow::error::ContextError::new());
            }
            // Must not be duplicated
            if parser_input.state.seen_meta_tags.contains(&name) {
                parser_input.state.errors.push(ParseError::new(
                    ErrorKind::SvelteMetaDuplicate,
                    Span::new(name_start, name_end),
                    format!("A component can only have one `<{}>` element", name),
                ));
                return Err(winnow::error::ContextError::new());
            }
            parser_input.state.seen_meta_tags.insert(name.clone());
        }
    }

    // parse attributes (in loose mode, might stop at EOF or unexpected tokens)
    let attributes = if loose {
        parse_attributes_loose(parser_input)?
    } else {
        parse_attributes(parser_input)?
    };

    // Check if this element has shadowrootmode attribute
    let has_shadowrootmode = attributes
        .iter()
        .any(|attr| matches!(attr, AttributeNode::Attribute(a) if a.name == "shadowrootmode"));

    // check self-closing /> or >
    let self_closing = opt(literal("/")).parse_next(parser_input)?.is_some();
    let got_close = if loose {
        opt(literal(">")).parse_next(parser_input)?.is_some()
    } else {
        literal(">").parse_next(parser_input)?;
        true
    };

    // Check shadowroot context BEFORE parsing children (for nested slot detection)
    let parent_is_shadowroot = parser_input.state.parent_is_shadowroot_template();

    let fragment = if self_closing || is_void_element(&name) || !got_close {
        Fragment { nodes: vec![] }
    } else {
        // Push this element onto stack before parsing children
        parser_input.state.push_element(has_shadowrootmode);

        let nodes = if loose {
            parse_children_loose(parser_input, &name)?
        } else {
            let (nodes, _): (Vec<FragmentNode>, _) =
                repeat_till(0.., fragment_node_parser, closing_tag_parser(&name))
                    .parse_next(parser_input)?;
            nodes
        };

        // Pop element from stack after parsing children
        parser_input.state.pop_element();

        Fragment { nodes }
    };

    let end = parser_input.input.previous_token_end();
    let span = Span::new(start, end);

    Ok(classify_element(
        &name,
        name_loc,
        attributes,
        fragment,
        span,
        parent_is_shadowroot || has_shadowrootmode,
        loose,
    ))
}

/// Parse children in loose mode: stop at own closing tag, parent closing tag, or EOF.
fn parse_children_loose(
    parser_input: &mut ParserInput,
    name: &str,
) -> ParseResult<Vec<FragmentNode>> {
    let mut nodes = Vec::new();

    loop {
        // Check for our own closing tag
        if opt(peek(closing_tag_peek(name)))
            .parse_next(parser_input)?
            .is_some()
        {
            // Consume the closing tag
            closing_tag_parser(name).parse_next(parser_input)?;
            return Ok(nodes);
        }

        // Check for EOF
        if parser_input.input.is_empty() {
            return Ok(nodes);
        }

        // Check for a parent's closing tag (any </...> that's not ours)
        if opt(peek(literal("</"))).parse_next(parser_input)?.is_some() {
            // This is a parent's closing tag, stop here without consuming
            return Ok(nodes);
        }

        // Check for block close ({/...}) that might indicate parent boundary
        if opt(peek(literal("{/"))).parse_next(parser_input)?.is_some() {
            return Ok(nodes);
        }

        // Try to parse a node
        match fragment_node_parser.parse_next(parser_input) {
            Ok(node) => nodes.push(node),
            Err(_) => {
                // Can't parse further, stop
                return Ok(nodes);
            }
        }
    }
}

/// Peek for a closing tag without consuming.
fn closing_tag_peek<'a, 'i>(
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

/// Parse tag name in loose mode: allows incomplete names ending with '.'
fn tag_name_parser_loose<'i>(parser_input: &mut ParserInput<'i>) -> ParseResult<String> {
    let name = take_while(1.., |c: char| {
        c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == ':'
    })
    .parse_next(parser_input)?;
    Ok(name.to_string())
}

/// Parse attributes in loose mode: stops at EOF, > or / like normal,
/// but also stops if attribute parsing fails.
fn parse_attributes_loose(parser_input: &mut ParserInput) -> ParseResult<Vec<AttributeNode>> {
    let mut attributes = Vec::new();
    loop {
        take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;

        // Check for EOF
        if parser_input.input.is_empty() {
            break;
        }

        // stop if we hit > or />
        if opt(peek(literal(">"))).parse_next(parser_input)?.is_some() {
            break;
        }
        if opt(peek(literal("/"))).parse_next(parser_input)?.is_some() {
            break;
        }

        // stop if we hit < (start of new tag) or {/ or {# or {: (block boundaries)
        let remaining: &str = &parser_input.input;
        if remaining.starts_with('<')
            || remaining.starts_with("{/")
            || remaining.starts_with("{#")
            || remaining.starts_with("{:")
        {
            break;
        }

        match attribute_parser(parser_input) {
            Ok(attr) => attributes.push(attr),
            Err(_) => break, // Can't parse attribute, stop
        }
    }
    Ok(attributes)
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
    name_loc: SourceLocation,
    attributes: Vec<AttributeNode>,
    fragment: Fragment,
    span: Span,
    is_inside_shadowroot: bool,
    loose: bool,
) -> FragmentNode {
    match name {
        "svelte:head" => FragmentNode::SvelteHead(SvelteHead {
            span,
            name: name.to_string(),
            name_loc,
            attributes,
            fragment,
        }),
        "svelte:body" => FragmentNode::SvelteBody(SvelteBody {
            span,
            name: name.to_string(),
            name_loc,
            attributes,
            fragment,
        }),
        "svelte:window" => FragmentNode::SvelteWindow(SvelteWindow {
            span,
            name: name.to_string(),
            name_loc,
            attributes,
            fragment,
        }),
        "svelte:document" => FragmentNode::SvelteDocument(SvelteDocument {
            span,
            name: name.to_string(),
            name_loc,
            attributes,
            fragment,
        }),
        "svelte:fragment" => FragmentNode::SvelteFragment(SvelteFragment {
            span,
            name: name.to_string(),
            name_loc,
            attributes,
            fragment,
        }),
        "svelte:boundary" => FragmentNode::SvelteBoundary(SvelteBoundary {
            span,
            name: name.to_string(),
            name_loc,
            attributes,
            fragment,
        }),
        "svelte:self" => FragmentNode::SvelteSelf(SvelteSelf {
            span,
            name: name.to_string(),
            name_loc,
            attributes,
            fragment,
        }),
        "svelte:element" => {
            let (this_expr, remaining_attrs) = extract_this_attribute(attributes);
            let tag = match this_expr {
                Some(expr) => expr,
                None => {
                    JsNode(serde_json::json!({"type": "Identifier", "name": "", "start": span.start, "end": span.start}))
                }
            };
            FragmentNode::SvelteElement(SvelteElement {
                span,
                name: name.to_string(),
                name_loc,
                tag,
                attributes: remaining_attrs,
                fragment,
            })
        }
        "svelte:component" => {
            let (this_expr, remaining_attrs) = extract_this_attribute(attributes);
            let expression = match this_expr {
                Some(expr) => expr,
                None => {
                    JsNode(serde_json::json!({"type": "Identifier", "name": "", "start": span.start, "end": span.start}))
                }
            };
            FragmentNode::SvelteComponent(SvelteComponent {
                span,
                name: name.to_string(),
                name_loc,
                expression,
                attributes: remaining_attrs,
                fragment,
            })
        }
        "svelte:options" => FragmentNode::SvelteOptionsRaw(SvelteOptionsRaw {
            span,
            name_loc,
            attributes,
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
            name: name.to_string(),
            name_loc,
            attributes,
            fragment,
        }),
        _ => {
            if name.starts_with(char::is_uppercase)
                || (name.contains('.') && !name.ends_with('.'))
                || (loose && name.ends_with('.'))
            {
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

/// Extract the `this` attribute from an attribute list.
/// Returns (expression from `this`, remaining attributes without `this`).
fn extract_this_attribute(mut attributes: Vec<AttributeNode>) -> (Option<JsNode>, Vec<AttributeNode>) {
    let this_idx = attributes.iter().position(|attr| {
        matches!(attr, AttributeNode::Attribute(a) if a.name == "this")
    });

    let expr = if let Some(idx) = this_idx {
        let attr_node = attributes.remove(idx);
        if let AttributeNode::Attribute(a) = attr_node {
            match a.value {
                AttributeValue::Expression(tag) => Some(tag.expression),
                AttributeValue::Sequence(items) => {
                    if items.len() == 1 {
                        match &items[0] {
                            AttributeSequenceValue::Text(text) => {
                                // Text value → create a Literal node
                                Some(JsNode(serde_json::json!({
                                    "type": "Literal",
                                    "start": text.span.start,
                                    "end": text.span.end,
                                    "value": text.data,
                                    "raw": format!("\"{}\"", text.data)
                                })))
                            }
                            AttributeSequenceValue::ExpressionTag(tag) => {
                                // Expression inside quotes → extract directly
                                Some(tag.expression.clone())
                            }
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            None
        }
    } else {
        None
    };

    (expr, attributes)
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
