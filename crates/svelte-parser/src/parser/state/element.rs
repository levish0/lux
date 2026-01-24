use std::sync::LazyLock;

use regex::Regex;
use svelte_ast::attributes::{Attribute, AttributeSequenceValue, AttributeValue, SpreadAttribute};
use svelte_ast::elements::{
    Component, RegularElement, SlotElement, SvelteBody, SvelteBoundary, SvelteComponent,
    SvelteDocument, SvelteElement, SvelteFragment, SvelteHead, SvelteOptionsRaw, SvelteSelf,
    SvelteWindow, TitleElement,
};
use svelte_ast::metadata::ExpressionNodeMetadata;
use svelte_ast::node::{AttributeNode, FragmentNode};
use svelte_ast::root::{Fragment, ScriptContext};
use svelte_ast::span::Span;
use svelte_ast::tags::{AttachTag, ExpressionTag};
use svelte_ast::text::{Comment, Text};
use oxc_allocator::Allocator;

use crate::parser::{ParseError, Parser, StackFrame};
use crate::parser::html_entities::decode_character_references;
use crate::parser::read::expression::read_expression;
use crate::parser::read::script::read_script;
use crate::parser::read::style::read_style;

/// Reference: regex_invalid_unquoted_attribute_value = /^(\/>|[\s"'=<>`])/
static REGEX_INVALID_UNQUOTED_ATTRIBUTE_VALUE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^(/>|[\s"'=<>`])"#).unwrap());

/// Reference: regex_closing_comment = /-->/
static REGEX_CLOSING_COMMENT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"-->").unwrap());

/// Reference: regex_whitespace_or_slash_or_closing_tag = /(\s|\/|>)/
static REGEX_WHITESPACE_OR_SLASH_OR_CLOSING_TAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(\s|/|>)").unwrap());

/// Reference: regex_token_ending_character = /[\s=/>"']/
static REGEX_TOKEN_ENDING_CHARACTER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"[\s=/>"']"#).unwrap());

/// Reference: regex_starts_with_quote_characters = /^["']/
static REGEX_STARTS_WITH_QUOTE_CHARACTERS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^["']"#).unwrap());

/// Reference: regex_attribute_value = /^(?:"([^"]*)"|'([^'])*'|([^>\s]+))/
static REGEX_ATTRIBUTE_VALUE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^(?:"([^"]*)"|'([^'])*'|([^>\s]+))"#).unwrap());

/// Reference: regex_closing_textarea_tag = /^<\/textarea(\s[^>]*)?>/i
static REGEX_CLOSING_TEXTAREA_TAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^</textarea(\s[^>]*)?>").unwrap());

/// Reference: regex_valid_element_name
static REGEX_VALID_ELEMENT_NAME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:![a-zA-Z]+|[a-zA-Z](?:[a-zA-Z0-9-]*[a-zA-Z0-9])?|[a-zA-Z][a-zA-Z0-9]*:[a-zA-Z][a-zA-Z0-9-]*[a-zA-Z0-9])$")
        .unwrap()
});

/// Reference: regex_valid_component_name
/// Must start with uppercase letter (if no dots), or contain dots.
/// Uses Unicode-aware matching: \p{Lu} for uppercase start, \p{ID_Start}/\p{ID_Continue} for identifiers.
static REGEX_VALID_COMPONENT_NAME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:\p{Lu}[$\x{200c}\x{200d}\p{ID_Continue}.]*|\p{ID_Start}[$\x{200c}\x{200d}\p{ID_Continue}]*(?:\.[$\x{200c}\x{200d}\p{ID_Continue}]+)+)$")
        .unwrap()
});

/// Element state.
/// Matches reference: `state/element.js`
///
/// Handles `<...>` — opening tags, closing tags, comments, special elements.
pub fn element(parser: &mut Parser) -> Result<(), ParseError> {
    let start = parser.index;
    parser.index += 1; // skip `<`

    // Comment: <!-- ... -->
    if parser.match_str("!--") {
        parser.index += 3;
        read_comment(parser, start);
        return Ok(());
    }

    // Closing tag: </name>
    if parser.eat("/") {
        close_tag(parser)?;
        return Ok(());
    }

    // Opening tag
    open_tag(parser, start)?;
    Ok(())
}

/// Read an HTML comment: `<!-- ... -->`
fn read_comment(parser: &mut Parser, start: usize) {
    let data = parser.read_until(&REGEX_CLOSING_COMMENT).to_string();

    if parser.match_str("-->") {
        parser.index += 3; // consume -->
    }

    parser.append(FragmentNode::Comment(Comment {
        span: Span::new(start, parser.index),
        data,
    }));
}

/// Handle a closing tag: `</name>`
fn close_tag(parser: &mut Parser) -> Result<(), ParseError> {
    parser.allow_whitespace();
    let name = parser
        .read_until(&REGEX_WHITESPACE_OR_SLASH_OR_CLOSING_TAG)
        .to_string();
    parser.allow_whitespace();
    parser.eat_required(">")?;

    // Find matching open element on the stack
    let found = parser.stack.iter().rev().any(|frame| frame_name(frame) == Some(&name));

    if !found {
        if !parser.loose {
            return Err(parser.error(
                crate::error::ErrorKind::BlockUnexpectedClose,
                parser.index,
                format!("'</{name}>' has no matching open tag"),
            ));
        }
        return Ok(());
    }

    // Pop until we find the matching element
    loop {
        let (frame, fragment) = parser.pop();
        let Some(frame) = frame else { break };
        let fragment_nodes = fragment.unwrap_or_default();
        let is_match = frame_name(&frame) == Some(&name);
        let node = frame_to_node(frame, parser.index, fragment_nodes, parser.allocator);
        if let Some(node) = node {
            parser.append(node);
        }
        if is_match {
            break;
        }
    }

    Ok(())
}

/// Get the element name from a StackFrame (None for block frames).
fn frame_name<'a, 'b>(frame: &'b StackFrame<'a>) -> Option<&'b String> {
    match frame {
        StackFrame::RegularElement { name, .. }
        | StackFrame::Component { name, .. }
        | StackFrame::SvelteElement { name, .. }
        | StackFrame::SvelteComponent { name, .. }
        | StackFrame::SvelteSelf { name, .. }
        | StackFrame::SvelteHead { name, .. }
        | StackFrame::SvelteBody { name, .. }
        | StackFrame::SvelteWindow { name, .. }
        | StackFrame::SvelteDocument { name, .. }
        | StackFrame::SvelteFragment { name, .. }
        | StackFrame::SvelteOptions { name, .. }
        | StackFrame::TitleElement { name, .. }
        | StackFrame::SlotElement { name, .. }
        | StackFrame::SvelteBoundary { name, .. } => Some(name),
        _ => None,
    }
}

/// Convert a StackFrame + collected fragment nodes into a FragmentNode.
fn frame_to_node<'a>(
    frame: StackFrame<'a>,
    end: usize,
    nodes: Vec<FragmentNode<'a>>,
    allocator: &'a Allocator,
) -> Option<FragmentNode<'a>> {
    let fragment = Fragment { nodes };
    match frame {
        StackFrame::RegularElement { start, name, name_loc, attributes } => {
            Some(FragmentNode::RegularElement(RegularElement {
                span: Span::new(start, end), name, name_loc, attributes, fragment,
            }))
        }
        StackFrame::Component { start, name, name_loc, attributes } => {
            Some(FragmentNode::Component(Component {
                span: Span::new(start, end), name, name_loc, attributes, fragment,
            }))
        }
        StackFrame::SvelteElement { start, name, name_loc, tag, attributes } => {
            let tag = tag.unwrap_or_else(|| make_null_literal(allocator));
            Some(FragmentNode::SvelteElement(SvelteElement {
                span: Span::new(start, end), name, name_loc, tag, attributes, fragment,
            }))
        }
        StackFrame::SvelteComponent { start, name, name_loc, expression, attributes } => {
            let expression = expression.unwrap_or_else(|| make_null_literal(allocator));
            Some(FragmentNode::SvelteComponent(SvelteComponent {
                span: Span::new(start, end), name, name_loc, expression, attributes, fragment,
            }))
        }
        StackFrame::SvelteSelf { start, name, name_loc, attributes } => {
            Some(FragmentNode::SvelteSelf(SvelteSelf {
                span: Span::new(start, end), name, name_loc, attributes, fragment,
            }))
        }
        StackFrame::SvelteHead { start, name, name_loc, attributes } => {
            Some(FragmentNode::SvelteHead(SvelteHead {
                span: Span::new(start, end), name, name_loc, attributes, fragment,
            }))
        }
        StackFrame::SvelteBody { start, name, name_loc, attributes } => {
            Some(FragmentNode::SvelteBody(SvelteBody {
                span: Span::new(start, end), name, name_loc, attributes, fragment,
            }))
        }
        StackFrame::SvelteWindow { start, name, name_loc, attributes } => {
            Some(FragmentNode::SvelteWindow(SvelteWindow {
                span: Span::new(start, end), name, name_loc, attributes, fragment,
            }))
        }
        StackFrame::SvelteDocument { start, name, name_loc, attributes } => {
            Some(FragmentNode::SvelteDocument(SvelteDocument {
                span: Span::new(start, end), name, name_loc, attributes, fragment,
            }))
        }
        StackFrame::SvelteFragment { start, name, name_loc, attributes } => {
            Some(FragmentNode::SvelteFragment(SvelteFragment {
                span: Span::new(start, end), name, name_loc, attributes, fragment,
            }))
        }
        StackFrame::SvelteBoundary { start, name, name_loc, attributes } => {
            Some(FragmentNode::SvelteBoundary(SvelteBoundary {
                span: Span::new(start, end), name, name_loc, attributes, fragment,
            }))
        }
        StackFrame::SvelteOptions { start, name, name_loc, attributes } => {
            Some(FragmentNode::SvelteOptionsRaw(SvelteOptionsRaw {
                span: Span::new(start, end), name, name_loc, attributes, fragment,
            }))
        }
        StackFrame::TitleElement { start, name, name_loc, attributes } => {
            Some(FragmentNode::TitleElement(TitleElement {
                span: Span::new(start, end), name, name_loc, attributes, fragment,
            }))
        }
        StackFrame::SlotElement { start, name, name_loc, attributes } => {
            Some(FragmentNode::SlotElement(SlotElement {
                span: Span::new(start, end), name, name_loc, attributes, fragment,
            }))
        }
        _ => None, // Block frames — handled elsewhere
    }
}

/// Create a dummy NullLiteral expression (fallback for missing `this` in loose mode).
fn make_null_literal<'a>(allocator: &'a Allocator) -> oxc_ast::ast::Expression<'a> {
    oxc_ast::ast::Expression::NullLiteral(oxc_allocator::Box::new_in(
        oxc_ast::ast::NullLiteral { span: oxc_span::Span::new(0, 0) },
        allocator,
    ))
}

/// Handle an opening tag: `<name ...>`
/// Reference: element.js lines 60-421
fn open_tag(parser: &mut Parser, start: usize) -> Result<(), ParseError> {
    let name_start = parser.index;
    let name = parser
        .read_until(&REGEX_WHITESPACE_OR_SLASH_OR_CLOSING_TAG)
        .to_string();
    let name_end = parser.index;

    if name.is_empty() {
        // Not a valid tag — treat as text
        parser.index = start;
        super::text::text(parser);
        return Ok(());
    }
    let name_loc = parser.source_location(name_start, name_end);

    // Reference: element.js:219-222
    // Top-level script/style uses read_static_attribute; others use read_attribute.
    let is_top_level_script_or_style =
        (name == "script" || name == "style") && parser.stack.is_empty();

    let mut attributes = if is_top_level_script_or_style {
        read_static_attributes(parser)
    } else {
        read_attributes(parser)
    };

    parser.allow_whitespace();

    // Handle top-level <script> and <style> specially
    if is_top_level_script_or_style {
        parser.eat_required(">")?;

        if name == "script" {
            let script = read_script(parser, start, attributes)?;
            if script.context == ScriptContext::Module {
                parser.module = Some(script);
            } else {
                parser.instance = Some(script);
            }
        } else {
            let stylesheet = read_style(parser, start, attributes)?;
            parser.css = Some(stylesheet);
        }

        return Ok(());
    }

    // Extract `this` attribute for svelte:component / svelte:element
    let this_expression = if name == "svelte:component" || name == "svelte:element" {
        extract_this_attribute(&mut attributes, parser, &name, start)
    } else {
        None
    };

    let self_closing = parser.eat("/");
    parser.eat_required(">")?;

    let is_void = crate::parser::utils::is_void(&name);

    if self_closing || is_void {
        // Self-closing or void — no children, immediately append
        let node = make_element_node(
            &name, start, parser.index, name_loc.clone(),
            attributes, Fragment { nodes: Vec::new() }, this_expression, parser,
        );
        parser.append(node);
    } else if name == "textarea" {
        // Reference: element.js:391-399 — textarea reads content as sequence
        let nodes = read_sequence(parser, |p| {
            if p.index >= p.template.len() {
                return true;
            }
            p.match_regex(&REGEX_CLOSING_TEXTAREA_TAG).is_some()
        });
        parser.read(&REGEX_CLOSING_TEXTAREA_TAG);
        let fragment_nodes: Vec<FragmentNode> = nodes
            .into_iter()
            .map(|chunk| match chunk {
                AttributeSequenceValue::Text(t) => FragmentNode::Text(t),
                AttributeSequenceValue::ExpressionTag(et) => FragmentNode::ExpressionTag(et),
            })
            .collect();
        let node = make_element_node(
            &name, start, parser.index, name_loc.clone(),
            attributes, Fragment { nodes: fragment_nodes }, this_expression, parser,
        );
        parser.append(node);
    } else if name == "script" || name == "style" {
        // Reference: element.js:400-417 — non-top-level script/style reads raw text
        let content_start = parser.index;
        let closing_re =
            Regex::new(&format!(r"</{}>", regex::escape(&name))).unwrap();
        let data = parser.read_until(&closing_re).to_string();
        let content_end = parser.index;
        let closing_tag_re =
            Regex::new(&format!(r"^</{}\s*>", regex::escape(&name))).unwrap();
        parser.read(&closing_tag_re);

        let text_node = FragmentNode::Text(Text {
            span: Span::new(content_start, content_end),
            raw: data.clone(),
            data,
        });
        let node = make_element_node(
            &name, start, parser.index, name_loc.clone(),
            attributes, Fragment { nodes: vec![text_node] }, this_expression, parser,
        );
        parser.append(node);
    } else {
        // Normal element — push onto stack, children parsed in fragment loop
        let frame = make_stack_frame(
            &name, start, name_loc, attributes, this_expression, parser,
        );
        parser.stack.push(frame);
        parser.fragments.push(Vec::new());
    }

    Ok(())
}

/// Check if any ancestor on the stack is SvelteHead.
/// Reference: element.js lines 425-433
fn parent_is_head(stack: &[StackFrame]) -> bool {
    for frame in stack.iter().rev() {
        match frame {
            StackFrame::SvelteHead { .. } => return true,
            StackFrame::RegularElement { .. } | StackFrame::Component { .. } => return false,
            _ => {}
        }
    }
    false
}

/// Check if any ancestor is a RegularElement with `shadowrootmode` attribute.
/// Reference: element.js lines 436-450
fn parent_is_shadowroot_template(stack: &[StackFrame]) -> bool {
    for frame in stack.iter().rev() {
        if let StackFrame::RegularElement { attributes, .. } = frame {
            if attributes.iter().any(|a| {
                matches!(a, AttributeNode::Attribute(attr) if attr.name == "shadowrootmode")
            }) {
                return true;
            }
        }
    }
    false
}

/// Determine whether the name is a component name.
/// Reference: regex_valid_component_name test + loose dot-ending check.
fn is_component_name(name: &str, loose: bool) -> bool {
    REGEX_VALID_COMPONENT_NAME.is_match(name) || (loose && name.ends_with('.'))
}

/// Extract `this={expr}` from attributes for svelte:component / svelte:element.
/// Reference: element.js lines 255-311
fn extract_this_attribute<'a>(
    attributes: &mut Vec<AttributeNode<'a>>,
    parser: &mut Parser<'a>,
    name: &str,
    start: usize,
) -> Option<oxc_ast::ast::Expression<'a>> {
    let index = attributes.iter().position(|a| {
        matches!(a, AttributeNode::Attribute(attr) if attr.name == "this")
    });

    let Some(index) = index else {
        if !parser.loose {
            parser.error(
                crate::error::ErrorKind::ExpectedToken,
                start,
                format!("'<{name}>' requires a 'this' attribute"),
            );
        }
        return None;
    };

    let attr_node = attributes.remove(index);
    let AttributeNode::Attribute(attr) = attr_node else {
        return None;
    };

    match attr.value {
        AttributeValue::ExpressionTag(et) => Some(et.expression),
        AttributeValue::Sequence(chunks) => {
            if chunks.len() == 1 {
                let chunk = chunks.into_iter().next().unwrap();
                match chunk {
                    AttributeSequenceValue::ExpressionTag(et) => Some(et.expression),
                    AttributeSequenceValue::Text(t) => {
                        // Reference: text values become string literals for svelte:element
                        if name == "svelte:element" {
                            let raw = format!("'{}'", t.raw);
                            let raw_str = parser.allocator.alloc_str(&raw);
                            let val_str = parser.allocator.alloc_str(&t.data);
                            Some(oxc_ast::ast::Expression::StringLiteral(
                                oxc_allocator::Box::new_in(
                                    oxc_ast::ast::StringLiteral {
                                        span: oxc_span::Span::new(
                                            t.span.start as u32,
                                            t.span.end as u32,
                                        ),
                                        value: oxc_span::Atom::from(val_str as &str),
                                        raw: Some(oxc_span::Atom::from(raw_str as &str)),
                                        lone_surrogates: false,
                                    },
                                    parser.allocator,
                                ),
                            ))
                        } else {
                            None
                        }
                    }
                }
            } else {
                None
            }
        }
        AttributeValue::True => None,
    }
}

/// Create a FragmentNode directly from name + parts (for self-closing/void/textarea/script/style).
/// Uses name matching to determine element type (reference: element.js lines 157-166).
fn make_element_node<'a>(
    name: &str,
    start: usize,
    end: usize,
    name_loc: svelte_ast::span::SourceLocation,
    attributes: Vec<AttributeNode<'a>>,
    fragment: Fragment<'a>,
    this_expression: Option<oxc_ast::ast::Expression<'a>>,
    parser: &Parser<'a>,
) -> FragmentNode<'a> {
    let span = Span::new(start, end);
    let name_str = name.to_string();
    match name {
        "svelte:head" => FragmentNode::SvelteHead(SvelteHead {
            span, name: name_str, name_loc, attributes, fragment,
        }),
        "svelte:options" => FragmentNode::SvelteOptionsRaw(SvelteOptionsRaw {
            span, name: name_str, name_loc, attributes, fragment,
        }),
        "svelte:window" => FragmentNode::SvelteWindow(SvelteWindow {
            span, name: name_str, name_loc, attributes, fragment,
        }),
        "svelte:document" => FragmentNode::SvelteDocument(SvelteDocument {
            span, name: name_str, name_loc, attributes, fragment,
        }),
        "svelte:body" => FragmentNode::SvelteBody(SvelteBody {
            span, name: name_str, name_loc, attributes, fragment,
        }),
        "svelte:element" => {
            let tag = this_expression.unwrap_or_else(|| make_null_literal(parser.allocator));
            FragmentNode::SvelteElement(SvelteElement {
                span, name: name_str, name_loc, tag, attributes, fragment,
            })
        }
        "svelte:component" => {
            let expression = this_expression.unwrap_or_else(|| make_null_literal(parser.allocator));
            FragmentNode::SvelteComponent(SvelteComponent {
                span, name: name_str, name_loc, expression, attributes, fragment,
            })
        }
        "svelte:self" => FragmentNode::SvelteSelf(SvelteSelf {
            span, name: name_str, name_loc, attributes, fragment,
        }),
        "svelte:fragment" => FragmentNode::SvelteFragment(SvelteFragment {
            span, name: name_str, name_loc, attributes, fragment,
        }),
        "svelte:boundary" => FragmentNode::SvelteBoundary(SvelteBoundary {
            span, name: name_str, name_loc, attributes, fragment,
        }),
        _ if is_component_name(name, parser.loose) => FragmentNode::Component(Component {
            span, name: name_str, name_loc, attributes, fragment,
        }),
        "title" if parent_is_head(&parser.stack) => FragmentNode::TitleElement(TitleElement {
            span, name: name_str, name_loc, attributes, fragment,
        }),
        "slot" if !parent_is_shadowroot_template(&parser.stack) => {
            FragmentNode::SlotElement(SlotElement {
                span, name: name_str, name_loc, attributes, fragment,
            })
        }
        _ => FragmentNode::RegularElement(RegularElement {
            span, name: name_str, name_loc, attributes, fragment,
        }),
    }
}

/// Create a StackFrame from name + parts (for elements with children).
/// Uses name matching to determine element type (reference: element.js lines 157-166).
fn make_stack_frame<'a>(
    name: &str,
    start: usize,
    name_loc: svelte_ast::span::SourceLocation,
    attributes: Vec<AttributeNode<'a>>,
    this_expression: Option<oxc_ast::ast::Expression<'a>>,
    parser: &Parser<'a>,
) -> StackFrame<'a> {
    let name_str = name.to_string();
    match name {
        "svelte:head" => StackFrame::SvelteHead {
            start, name: name_str, name_loc, attributes,
        },
        "svelte:options" => StackFrame::SvelteOptions {
            start, name: name_str, name_loc, attributes,
        },
        "svelte:window" => StackFrame::SvelteWindow {
            start, name: name_str, name_loc, attributes,
        },
        "svelte:document" => StackFrame::SvelteDocument {
            start, name: name_str, name_loc, attributes,
        },
        "svelte:body" => StackFrame::SvelteBody {
            start, name: name_str, name_loc, attributes,
        },
        "svelte:element" => StackFrame::SvelteElement {
            start, name: name_str, name_loc, tag: this_expression, attributes,
        },
        "svelte:component" => StackFrame::SvelteComponent {
            start, name: name_str, name_loc, expression: this_expression, attributes,
        },
        "svelte:self" => StackFrame::SvelteSelf {
            start, name: name_str, name_loc, attributes,
        },
        "svelte:fragment" => StackFrame::SvelteFragment {
            start, name: name_str, name_loc, attributes,
        },
        "svelte:boundary" => StackFrame::SvelteBoundary {
            start, name: name_str, name_loc, attributes,
        },
        _ if is_component_name(name, parser.loose) => StackFrame::Component {
            start, name: name_str, name_loc, attributes,
        },
        "title" if parent_is_head(&parser.stack) => StackFrame::TitleElement {
            start, name: name_str, name_loc, attributes,
        },
        "slot" if !parent_is_shadowroot_template(&parser.stack) => StackFrame::SlotElement {
            start, name: name_str, name_loc, attributes,
        },
        _ => StackFrame::RegularElement {
            start, name: name_str, name_loc, attributes,
        },
    }
}

/// Read static attributes (for top-level script/style tags).
/// Port of reference `read_static_attribute` in element.js.
fn read_static_attributes<'a>(parser: &mut Parser<'a>) -> Vec<AttributeNode<'a>> {
    let mut attributes = Vec::new();

    loop {
        parser.allow_whitespace();
        if parser.index >= parser.template.len() || parser.match_str(">") {
            break;
        }
        if let Some(attr) = read_static_attribute(parser) {
            attributes.push(attr);
        } else {
            break;
        }
    }

    attributes
}

/// Read a single static attribute (name="value" or name).
/// Used for script/style tags where only simple attributes are valid.
/// Port of reference `read_static_attribute` in element.js.
fn read_static_attribute<'a>(parser: &mut Parser<'a>) -> Option<AttributeNode<'a>> {
    let start = parser.index;

    // Read attribute name
    let name = parser.read_until(&REGEX_TOKEN_ENDING_CHARACTER).to_string();
    if name.is_empty() {
        return None;
    }

    let name_loc = parser.source_location(start, parser.index);

    let value = if parser.eat("=") {
        parser.allow_whitespace();

        // Use regex_attribute_value to match the entire value
        let raw_match = parser.match_regex(&REGEX_ATTRIBUTE_VALUE);
        let Some(raw_full) = raw_match else {
            // No valid attribute value
            return None;
        };
        let raw_full = raw_full.to_string();
        parser.index += raw_full.len();

        let quoted = raw_full.starts_with('"') || raw_full.starts_with('\'');
        let raw = if quoted {
            &raw_full[1..raw_full.len() - 1]
        } else {
            &raw_full[..]
        };

        let val_start = parser.index - raw.len() - if quoted { 1 } else { 0 };
        let val_end = if quoted { parser.index - 1 } else { parser.index };
        let data = decode_character_references(raw, true);

        AttributeValue::Sequence(vec![AttributeSequenceValue::Text(Text {
            span: Span::new(val_start, val_end),
            raw: raw.to_string(),
            data,
        })])
    } else {
        if parser.match_regex(&REGEX_STARTS_WITH_QUOTE_CHARACTERS).is_some() && !parser.loose {
            return None; // Error: expected '='
        }
        AttributeValue::True
    };

    Some(AttributeNode::Attribute(Attribute {
        span: Span::new(start, parser.index),
        name,
        name_loc: Some(name_loc),
        value,
    }))
}

/// Read attributes until `>` or `/>`.
/// Port of reference `read_attribute` loop in element.js.
fn read_attributes<'a>(parser: &mut Parser<'a>) -> Vec<AttributeNode<'a>> {
    let mut attributes = Vec::new();

    loop {
        parser.allow_whitespace();

        if parser.index >= parser.template.len() {
            break;
        }

        if parser.match_str(">") || parser.match_str("/>") {
            break;
        }

        if let Some(attr) = read_attribute(parser) {
            attributes.push(attr);
        } else {
            // Could not read an attribute; skip a character to avoid infinite loop
            if parser.index < parser.template.len()
                && !parser.match_str(">")
                && !parser.match_str("/>")
            {
                parser.index += 1;
            } else {
                break;
            }
        }
    }

    attributes
}

/// Read a single attribute (or spread, or shorthand).
/// Port of reference `read_attribute` in element.js.
fn read_attribute<'a>(parser: &mut Parser<'a>) -> Option<AttributeNode<'a>> {
    let start = parser.index;

    // Handle `{...}` — attach, spread, or shorthand
    if parser.eat("{") {
        parser.allow_whitespace();

        // {@attach expr}
        if parser.eat("@attach") {
            parser.require_whitespace().ok();
            let expression = match read_expression(parser) {
                Ok(expr) => expr,
                Err(_) => {
                    skip_to_closing_brace_attr(parser);
                    return None;
                }
            };
            parser.allow_whitespace();
            parser.eat_required("}").ok();

            return Some(AttributeNode::AttachTag(AttachTag {
                span: Span::new(start, parser.index),
                expression,
            }));
        }

        // Spread attribute: {...expr}
        if parser.eat("...") {
            let expression = match read_expression(parser) {
                Ok(expr) => expr,
                Err(_) => {
                    skip_to_closing_brace_attr(parser);
                    return None;
                }
            };
            parser.allow_whitespace();
            parser.eat_required("}").ok();

            return Some(AttributeNode::SpreadAttribute(SpreadAttribute {
                span: Span::new(start, parser.index),
                expression,
            }));
        }

        // Shorthand: {name}
        let (id_name, id_start, id_end) = parser.read_identifier();
        if id_name.is_empty() {
            // Invalid or block-related — skip in loose mode
            if parser.loose
                && (parser.match_str("#")
                    || parser.match_str("/")
                    || parser.match_str("@")
                    || parser.match_str(":"))
            {
                parser.index = start; // restore
                return None;
            }
            skip_to_closing_brace_attr(parser);
            return None;
        }
        let id_name_str = id_name.to_string();

        parser.allow_whitespace();
        parser.eat_required("}").ok();

        // Create identifier expression for the shorthand
        let expression = make_identifier(parser, id_name, id_start, id_end);

        let expr_tag = ExpressionTag {
            span: Span::new(id_start, id_end),
            expression,
            metadata: ExpressionNodeMetadata::default(),
        };

        let name_loc = parser.source_location(id_start, id_end);

        return Some(AttributeNode::Attribute(Attribute {
            span: Span::new(start, parser.index),
            name: id_name_str,
            name_loc: Some(name_loc),
            value: AttributeValue::ExpressionTag(expr_tag),
        }));
    }

    // Read attribute name — consume until whitespace, =, /, >, ", '
    let name_start = parser.index;
    let name = read_attribute_name(parser);
    let name_end = parser.index;

    if name.is_empty() {
        return None;
    }

    let name_loc = parser.source_location(name_start, name_end);

    let mut end = parser.index;

    parser.allow_whitespace();

    // Check for directive type before reading value
    let colon_idx = name.find(':');
    let is_directive = colon_idx
        .map(|i| is_directive_prefix(&name[..i]))
        .unwrap_or(false);

    // Read value
    let value = if parser.eat("=") {
        parser.allow_whitespace();

        // Edge case: value=/>  (the '/' is the value, not self-closing)
        if parser.match_str("/")
            && parser.template.get(parser.index + 1..parser.index + 2) == Some(">")
        {
            let char_start = parser.index;
            parser.index += 1; // consume '/'
            end = parser.index;
            AttributeValue::Sequence(vec![AttributeSequenceValue::Text(Text {
                span: Span::new(char_start, char_start + 1),
                raw: "/".to_string(),
                data: "/".to_string(),
            })])
        } else {
            let v = read_attribute_value(parser);
            end = parser.index;
            v
        }
    } else if parser.match_regex(&REGEX_STARTS_WITH_QUOTE_CHARACTERS).is_some() {
        // Quote without '=' — error in strict mode (reference: e.expected_token)
        if !parser.loose {
            return None; // Would be an error, skip attribute
        }
        AttributeValue::True
    } else {
        AttributeValue::True
    };

    // Directive handling (name contains ':' with valid prefix)
    if let Some(colon_idx) = colon_idx {
        if is_directive {
            let prefix = &name[..colon_idx];
            let directive_name = &name[colon_idx + 1..];

            // Split modifiers by '|'
            let parts: Vec<&str> = directive_name.split('|').collect();
            let dir_name = parts[0].to_string();
            let modifiers: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

            return build_directive(
                parser, prefix, &dir_name, &modifiers, value, name_loc, start, end,
            );
        }
    }

    Some(AttributeNode::Attribute(Attribute {
        span: Span::new(start, end),
        name,
        name_loc: Some(name_loc),
        value,
    }))
}

/// Read an attribute/tag name: consume until regex_token_ending_character.
/// Port of reference `read_tag(parser, regex_token_ending_character)`.
fn read_attribute_name(parser: &mut Parser) -> String {
    parser.read_until(&REGEX_TOKEN_ENDING_CHARACTER).to_string()
}

/// Read an attribute value after `=`.
/// Port of reference `read_attribute_value` in element.js.
///
/// Returns:
/// - `ExpressionTag` for single `{expr}` without quotes
/// - `Sequence` for quoted values or multi-part values
fn read_attribute_value<'a>(parser: &mut Parser<'a>) -> AttributeValue<'a> {
    let quote_mark: Option<u8> = if parser.eat("'") {
        Some(b'\'')
    } else if parser.eat("\"") {
        Some(b'"')
    } else {
        None
    };

    // Empty quoted value: "" or ''
    if let Some(q) = quote_mark {
        if parser.index < parser.template.len() && parser.template.as_bytes()[parser.index] == q {
            parser.index += 1; // consume closing quote
            let pos = parser.index - 1;
            return AttributeValue::Sequence(vec![AttributeSequenceValue::Text(Text {
                span: Span::new(pos, pos),
                raw: String::new(),
                data: String::new(),
            })]);
        }
    }

    // Read sequence until done condition
    let chunks = if let Some(q) = quote_mark {
        read_sequence(parser, move |p| {
            p.index < p.template.len() && p.template.as_bytes()[p.index] == q
        })
    } else {
        // Unquoted: stop at regex_invalid_unquoted_attribute_value
        read_sequence(parser, |p| {
            if p.index >= p.template.len() {
                return true;
            }
            p.match_regex(&REGEX_INVALID_UNQUOTED_ATTRIBUTE_VALUE).is_some()
        })
    };

    // Consume closing quote
    if quote_mark.is_some() {
        parser.index += 1;
    }

    if chunks.is_empty() {
        return AttributeValue::True;
    }

    // Reference logic for return type:
    // if (quote_mark || value.length > 1 || value[0].type === 'Text') → return array (Sequence)
    // else → return value[0] (single ExpressionTag)
    if quote_mark.is_some() || chunks.len() > 1 {
        return AttributeValue::Sequence(chunks);
    }

    // Single chunk, no quotes
    let chunk = chunks.into_iter().next().unwrap();
    match chunk {
        AttributeSequenceValue::Text(t) => {
            AttributeValue::Sequence(vec![AttributeSequenceValue::Text(t)])
        }
        AttributeSequenceValue::ExpressionTag(et) => AttributeValue::ExpressionTag(et),
    }
}

/// Build a directive node from parsed attribute parts.
/// Reference: element.js lines 619-693
fn build_directive<'a>(
    parser: &mut Parser<'a>,
    prefix: &str,
    dir_name: &str,
    modifiers: &[String],
    value: AttributeValue<'a>,
    name_loc: svelte_ast::span::SourceLocation,
    start: usize,
    end: usize,
) -> Option<AttributeNode<'a>> {
    use svelte_ast::attributes::*;

    let span = Span::new(start, end);

    // StyleDirective gets the full value (can be true, ExpressionTag, or Sequence)
    if prefix == "style" {
        return Some(AttributeNode::StyleDirective(StyleDirective {
            span,
            name: dir_name.to_string(),
            name_loc: Some(name_loc),
            value,
            modifiers: modifiers.to_vec(),
        }));
    }

    // For other directives, extract expression from value
    // Reference logic:
    //   value === true → expression = null
    //   value is ExpressionTag → expression = value.expression
    //   value is Array → if len==1 && first is ExpressionTag → expression = first.expression
    //                     else → error (text in directive value)
    let expression = match value {
        AttributeValue::True => None,
        AttributeValue::ExpressionTag(et) => Some(et.expression),
        AttributeValue::Sequence(chunks) => {
            // Try to extract single ExpressionTag from sequence
            if chunks.len() == 1 {
                let chunk = chunks.into_iter().next().unwrap();
                match chunk {
                    AttributeSequenceValue::ExpressionTag(et) => Some(et.expression),
                    _ => None, // Text in directive → would be error, but skip in loose
                }
            } else {
                None // Multiple chunks in directive value → would be error
            }
        }
    };

    // If no expression but it's bind or class, create implicit identifier
    let expression = expression.or_else(|| {
        if prefix == "bind" || prefix == "class" {
            let id_start = start + prefix.len() + 1; // after "prefix:"
            let id_end = id_start + dir_name.len();
            Some(make_identifier(parser, dir_name, id_start, id_end))
        } else {
            None
        }
    });

    match prefix {
        "on" => Some(AttributeNode::OnDirective(OnDirective {
            span,
            name: dir_name.to_string(),
            name_loc: Some(name_loc),
            expression,
            modifiers: modifiers.to_vec(),
        })),
        "bind" => Some(AttributeNode::BindDirective(BindDirective {
            span,
            name: dir_name.to_string(),
            name_loc: Some(name_loc),
            expression: expression.unwrap_or_else(|| {
                make_identifier(parser, dir_name, start, end)
            }),
            modifiers: modifiers.to_vec(),
        })),
        "class" => Some(AttributeNode::ClassDirective(ClassDirective {
            span,
            name: dir_name.to_string(),
            name_loc: Some(name_loc),
            expression: expression.unwrap_or_else(|| {
                make_identifier(parser, dir_name, start, end)
            }),
            modifiers: modifiers.to_vec(),
        })),
        "use" => Some(AttributeNode::UseDirective(UseDirective {
            span,
            name: dir_name.to_string(),
            name_loc: Some(name_loc),
            expression,
            modifiers: modifiers.to_vec(),
        })),
        "animate" => Some(AttributeNode::AnimateDirective(AnimateDirective {
            span,
            name: dir_name.to_string(),
            name_loc: Some(name_loc),
            expression,
            modifiers: modifiers.to_vec(),
        })),
        "transition" | "in" | "out" => {
            Some(AttributeNode::TransitionDirective(TransitionDirective {
                span,
                name: dir_name.to_string(),
                name_loc: Some(name_loc),
                expression,
                modifiers: modifiers.to_vec(),
                intro: prefix == "in" || prefix == "transition",
                outro: prefix == "out" || prefix == "transition",
            }))
        }
        "let" => Some(AttributeNode::LetDirective(LetDirective {
            span,
            name: dir_name.to_string(),
            name_loc: Some(name_loc),
            expression,
            modifiers: modifiers.to_vec(),
        })),
        _ => {
            // Unknown directive prefix — shouldn't reach here due to is_directive_prefix check
            Some(AttributeNode::Attribute(Attribute {
                span,
                name: format!("{}:{}", prefix, dir_name),
                name_loc: Some(name_loc),
                value: AttributeValue::True,
            }))
        }
    }
}

/// Create an Identifier expression for implicit directive names.
fn make_identifier<'a>(
    parser: &Parser<'a>,
    name: &str,
    start: usize,
    end: usize,
) -> oxc_ast::ast::Expression<'a> {
    use std::cell::Cell;
    // Allocate name in the arena so it has 'a lifetime
    let name_str = parser.allocator.alloc_str(name);
    oxc_ast::ast::Expression::Identifier(oxc_allocator::Box::new_in(
        oxc_ast::ast::IdentifierReference {
            span: oxc_span::Span::new(start as u32, end as u32),
            name: oxc_span::Atom::from(name_str as &str),
            reference_id: Cell::new(None),
        },
        parser.allocator,
    ))
}

/// Skip to closing `}` for attribute expressions.
fn skip_to_closing_brace_attr(parser: &mut Parser) {
    let mut depth = 1u32;
    while parser.index < parser.template.len() && depth > 0 {
        let ch = parser.template.as_bytes()[parser.index];
        match ch {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    parser.index += 1;
                    return;
                }
            }
            b'\'' | b'"' | b'`' => {
                skip_string(parser, ch);
                continue;
            }
            _ => {}
        }
        parser.index += 1;
    }
}

/// Skip a string literal (single, double, or template).
fn skip_string(parser: &mut Parser, quote: u8) {
    parser.index += 1;
    while parser.index < parser.template.len() {
        let ch = parser.template.as_bytes()[parser.index];
        if ch == b'\\' {
            parser.index += 1;
        } else if ch == quote {
            return;
        } else if quote == b'`' && ch == b'$' {
            if parser.index + 1 < parser.template.len()
                && parser.template.as_bytes()[parser.index + 1] == b'{'
            {
                parser.index += 2;
                let mut depth = 1u32;
                while parser.index < parser.template.len() && depth > 0 {
                    let c = parser.template.as_bytes()[parser.index];
                    match c {
                        b'{' => depth += 1,
                        b'}' => depth -= 1,
                        _ => {}
                    }
                    if depth > 0 {
                        parser.index += 1;
                    }
                }
                if depth == 0 {
                    parser.index += 1;
                }
                continue;
            }
        }
        parser.index += 1;
    }
}

/// Check if a prefix is a valid directive prefix.
/// Reference: get_directive_type in element.js
fn is_directive_prefix(prefix: &str) -> bool {
    matches!(
        prefix,
        "on" | "bind" | "class" | "style" | "use" | "animate" | "transition" | "in" | "out"
            | "let"
    )
}

/// Read a sequence of Text and ExpressionTag chunks.
/// Port of reference `read_sequence` in element.js.
///
/// `done` is a closure that returns true when reading should stop.
fn read_sequence<'a>(
    parser: &mut Parser<'a>,
    done: impl Fn(&Parser<'a>) -> bool,
) -> Vec<AttributeSequenceValue<'a>> {
    let mut chunks: Vec<AttributeSequenceValue<'a>> = Vec::new();
    let mut text_start = parser.index;
    let mut raw = String::new();

    while parser.index < parser.template.len() {
        if done(parser) {
            // Flush any pending text
            if !raw.is_empty() {
                let data =
                    decode_character_references(&raw, true);
                chunks.push(AttributeSequenceValue::Text(Text {
                    span: Span::new(text_start, parser.index),
                    raw: raw.clone(),
                    data,
                }));
            }
            return chunks;
        }

        if parser.eat("{") {
            // Flush pending text
            if !raw.is_empty() {
                let data =
                    decode_character_references(&raw, true);
                chunks.push(AttributeSequenceValue::Text(Text {
                    span: Span::new(text_start, parser.index - 1),
                    raw: raw.clone(),
                    data,
                }));
                raw.clear();
            }

            let expr_start = parser.index - 1; // include the `{`
            parser.allow_whitespace();
            let expression = match read_expression(parser) {
                Ok(expr) => expr,
                Err(_) => {
                    skip_to_closing_brace_attr(parser);
                    text_start = parser.index;
                    continue;
                }
            };
            parser.allow_whitespace();
            parser.eat_required("}").ok();

            chunks.push(AttributeSequenceValue::ExpressionTag(ExpressionTag {
                span: Span::new(expr_start, parser.index),
                expression,
                metadata: ExpressionNodeMetadata::default(),
            }));

            text_start = parser.index;
        } else {
            raw.push(parser.template.as_bytes()[parser.index] as char);
            parser.index += 1;
        }
    }

    // EOF — flush remaining text (loose mode)
    if !raw.is_empty() {
        let data = decode_character_references(&raw, true);
        chunks.push(AttributeSequenceValue::Text(Text {
            span: Span::new(text_start, parser.index),
            raw,
            data,
        }));
    }

    chunks
}
