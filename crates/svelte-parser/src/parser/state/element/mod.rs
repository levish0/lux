mod attribute;

use std::sync::LazyLock;

use svelte_ast::attributes::{AttributeSequenceValue, AttributeValue};
use svelte_ast::elements::{
    Component, RegularElement, SlotElement, SvelteBody, SvelteBoundary, SvelteComponent,
    SvelteDocument, SvelteElement, SvelteFragment, SvelteHead, SvelteOptionsRaw, SvelteSelf,
    SvelteWindow, TitleElement,
};
use svelte_ast::node::{AttributeNode, FragmentNode};
use svelte_ast::root::{Fragment, ScriptContext};
use svelte_ast::span::Span;
use svelte_ast::text::{Comment, Text};
use oxc_allocator::Allocator;
use regex::Regex;
use ErrorKind::{
    SvelteMetaDuplicate, SvelteMetaInvalidPlacement, SvelteMetaInvalidTag, TagInvalidName,
};
use crate::parser::read::script::read_script;
use crate::parser::read::style::read_style;
use crate::parser::utils::closing_tag_omitted;
use crate::parser::{LastAutoClosedTag, ParseError, Parser, StackFrame};

use attribute::{read_attributes, read_sequence, read_static_attributes};
use crate::error::ErrorKind;

/// Reference: regex_valid_element_name (keep as regex — complex alternation)
static REGEX_VALID_ELEMENT_NAME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:![a-zA-Z]+|[a-zA-Z](?:[a-zA-Z0-9-]*[a-zA-Z0-9])?|[a-zA-Z][a-zA-Z0-9]*:[a-zA-Z][a-zA-Z0-9-]*[a-zA-Z0-9])$")
        .unwrap()
});

/// Reference: regex_valid_component_name (keep as regex — Unicode properties)
static REGEX_VALID_COMPONENT_NAME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:\p{Lu}[$\x{200c}\x{200d}\p{ID_Continue}.]*|\p{ID_Start}[$\x{200c}\x{200d}\p{ID_Continue}]*(?:\.[$\x{200c}\x{200d}\p{ID_Continue}]+)+)$")
        .unwrap()
});

/// Byte predicate: matches /(\s|\/|>)/ — whitespace, '/', or '>'
#[inline]
fn is_whitespace_or_slash_or_closing_tag(ch: u8) -> bool {
    matches!(ch, b' ' | b'\t' | b'\r' | b'\n' | b'/' | b'>')
}

/// Check if remaining template at parser.index matches `</textarea\s*>` (case-insensitive).
fn match_closing_textarea_tag(parser: &Parser) -> bool {
    let remaining = parser.template[parser.index..].as_bytes();
    // Need at least "</textarea>"  = 11 chars
    if remaining.len() < 11 {
        return false;
    }
    if remaining[0] != b'<' || remaining[1] != b'/' {
        return false;
    }
    // Case-insensitive "textarea"
    let tag = &remaining[2..10];
    if !tag.eq_ignore_ascii_case(b"textarea") {
        return false;
    }
    // Skip optional whitespace + attributes before '>'
    let mut i = 10;
    if i < remaining.len() && remaining[i] == b'>' {
        return true;
    }
    // Optional: `(\s[^>]*)?>`
    if i < remaining.len() && (remaining[i] as char).is_ascii_whitespace() {
        while i < remaining.len() && remaining[i] != b'>' {
            i += 1;
        }
        return i < remaining.len() && remaining[i] == b'>';
    }
    false
}

/// Consume a closing textarea tag at current position.
/// Returns the consumed length, or 0 if not matched.
fn eat_closing_textarea_tag(parser: &mut Parser) -> usize {
    let start = parser.index;
    let remaining = parser.template[parser.index..].as_bytes();
    if remaining.len() < 11 {
        return 0;
    }
    if remaining[0] != b'<' || remaining[1] != b'/' {
        return 0;
    }
    let tag = &remaining[2..10];
    if !tag.eq_ignore_ascii_case(b"textarea") {
        return 0;
    }
    let mut i = 10;
    if i < remaining.len() && remaining[i] == b'>' {
        parser.index += i + 1;
        return parser.index - start;
    }
    if i < remaining.len() && (remaining[i] as char).is_ascii_whitespace() {
        while i < remaining.len() && remaining[i] != b'>' {
            i += 1;
        }
        if i < remaining.len() && remaining[i] == b'>' {
            parser.index += i + 1;
            return parser.index - start;
        }
    }
    0
}

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
    let data = parser.read_until_str("-->");

    // Reference: parser.eat('-->', true) — required
    if !parser.eat("-->") {
        if !parser.loose {
            parser.error(
                ErrorKind::ExpectedToken,
                parser.index,
                "Expected '-->'.".to_string(),
            );
        }
    }

    parser.append(FragmentNode::Comment(Comment {
        span: Span::new(start, parser.index),
        data,
    }));
}

/// Handle a closing tag: `</name>`
/// Reference: element.js lines 73-128
fn close_tag(parser: &mut Parser) -> Result<(), ParseError> {
    let start = parser.index - 2; // position of `<` (we already consumed `</`)

    let name = parser
        .read_until_char(is_whitespace_or_slash_or_closing_tag);
    parser.allow_whitespace();
    parser.eat_required(">")?;

    // Reference: if (is_void(name)) e.void_element_invalid_content(start)
    if crate::parser::utils::is_void(&name) {
        if !parser.loose {
            return Err(parser.error(
                ErrorKind::VoidElementInvalidContent,
                start,
                format!("`</{name}>` is a void element — it cannot have content"),
            ));
        }
    }

    // Find matching open element on the stack
    let found = parser.stack.iter().rev().any(|frame| frame_name(frame) == Some(&name));

    if !found {
        if !parser.loose {
            if let Some(ref lac) = parser.last_auto_closed_tag {
                if lac.tag == name {
                    let reason = lac.reason;
                    return Err(parser.error(
                        ErrorKind::ElementInvalidClosingTag,
                        start,
                        format!("`</{name}>` attempted to close element that was already automatically closed by `<{reason}>`"),
                    ));
                }
            }
            return Err(parser.error(
                ErrorKind::ElementInvalidClosingTag,
                start,
                format!("`</{name}>` attempted to close an element that was not open"),
            ));
        }
        return Ok(());
    }

    // Pop until we find the matching element
    // Reference: implicitly-closed elements get end = start (of closing tag)
    // The matched element gets end = parser.index (after `>`)
    loop {
        let is_match = parser
            .stack
            .last()
            .and_then(|f| frame_name(f))
            .map(|n| n == name)
            .unwrap_or(false);

        let (frame, fragment) = parser.pop();
        let Some(frame) = frame else { break };
        let fragment_nodes = fragment.unwrap_or_default();

        if is_match {
            // The matched element: end = parser.index
            let node = frame_to_node(frame, parser.index, fragment_nodes, parser.allocator);
            if let Some(node) = node {
                parser.append(node);
            }
            break;
        } else {
            // Implicitly closed: end = start (position of the closing tag `<`)
            let node = frame_to_node(frame, start, fragment_nodes, parser.allocator);
            if let Some(node) = node {
                parser.append(node);
            }
        }
    }

    // Reference: cleanup last_auto_closed_tag when stack depth decreases
    if let Some(ref lac) = parser.last_auto_closed_tag {
        if parser.stack.len() < lac.depth {
            parser.last_auto_closed_tag = None;
        }
    }

    Ok(())
}

/// Get the element name from a StackFrame (None for block frames).
fn frame_name<'a>(frame: &StackFrame<'a>) -> Option<&'a str> {
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
fn make_null_literal(allocator: &'_ Allocator) -> oxc_ast::ast::Expression<'_> {
    oxc_ast::ast::Expression::NullLiteral(oxc_allocator::Box::new_in(
        oxc_ast::ast::NullLiteral { span: oxc_span::Span::new(0, 0) },
        allocator,
    ))
}

/// Reference: element.js:35-41 — root-only meta tags
fn is_root_only_meta_tag(name: &str) -> bool {
    matches!(
        name,
        "svelte:head" | "svelte:options" | "svelte:window" | "svelte:document" | "svelte:body"
    )
}

/// Reference: element.js:44-51 — all valid svelte: meta tags
fn is_meta_tag(name: &str) -> bool {
    is_root_only_meta_tag(name)
        || matches!(
            name,
            "svelte:element"
                | "svelte:component"
                | "svelte:self"
                | "svelte:fragment"
                | "svelte:boundary"
        )
}

/// Handle an opening tag: `<name ...>`
/// Reference: element.js lines 60-421
fn open_tag(parser: &mut Parser, start: usize) -> Result<(), ParseError> {
    let name_start = parser.index;
    let name = parser
        .read_until_char(is_whitespace_or_slash_or_closing_tag);
    let name_end = parser.index;

    if name.is_empty() {
        // Not a valid tag — treat as text
        parser.index = start;
        super::text::text(parser);
        return Ok(());
    }
    let name_loc = parser.source_location(name_start, name_end);

    // Reference: element.js:132-135 — unknown svelte: tag
    if name.starts_with("svelte:") && !is_meta_tag(&name) {
        if !parser.loose {
            return Err(parser.error(
                SvelteMetaInvalidTag,
                start + 1,
                format!("`<{}>` is not a valid svelte: tag", name),
            ));
        }
    }

    // Reference: element.js:137-143 — invalid tag name
    if !REGEX_VALID_ELEMENT_NAME.is_match(&name)
        && !REGEX_VALID_COMPONENT_NAME.is_match(&name)
    {
        if !parser.loose || !name.ends_with('.') {
            return Err(parser.error(
                TagInvalidName,
                start + 1,
                format!("`<{}>` is not a valid element name", name),
            ));
        }
    }

    // Reference: element.js:145-155 — root-only meta tag checks
    if is_root_only_meta_tag(&name) {
        if parser.meta_tags.contains(name) {
            if !parser.loose {
                return Err(parser.error(
                    SvelteMetaDuplicate,
                    start,
                    format!("`<{}>` can only appear once in a document", name),
                ));
            }
        }

        if !parser.stack.is_empty() {
            if !parser.loose {
                return Err(parser.error(
                    SvelteMetaInvalidPlacement,
                    start,
                    format!("`<{}>` tags cannot be inside elements or blocks", name),
                ));
            }
        }

        parser.meta_tags.insert(name);
    }

    parser.allow_whitespace();

    // Reference: element.js:203-213 — implicit closing
    // If the parent is a RegularElement that should auto-close when this tag opens, pop it.
    // Need to extract values first to avoid borrow checker issues
    let should_auto_close = if let Some(StackFrame::RegularElement { name: parent_name, .. }) = parser.stack.last() {
        if closing_tag_omitted(parent_name, &name) {
            Some(*parent_name)
        } else {
            None
        }
    } else {
        None
    };
    
    if let Some(parent_name_copy) = should_auto_close {
        let (frame, fragment) = parser.pop();
        if let Some(frame) = frame {
            let fragment_nodes = fragment.unwrap_or_default();
            let node = frame_to_node(frame, start, fragment_nodes, parser.allocator);
            if let Some(node) = node {
                parser.append(node);
            }
        }
        parser.last_auto_closed_tag = Some(LastAutoClosedTag {
            tag: parent_name_copy,
            reason: name,
            depth: parser.stack.len(),
        });
    }

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
                if parser.module.is_some() && !parser.loose {
                    parser.error(
                        ErrorKind::ScriptDuplicate,
                        start,
                        "A component can only have one `<script module>` element".to_string(),
                    );
                }
                parser.module = Some(script);
            } else {
                if parser.instance.is_some() && !parser.loose {
                    parser.error(
                        ErrorKind::ScriptDuplicate,
                        start,
                        "A component can only have one instance-level `<script>` element".to_string(),
                    );
                }
                parser.instance = Some(script);
            }
        } else {
            // Reference: e.style_duplicate check
            if parser.css.is_some() && !parser.loose {
                parser.error(
                    ErrorKind::StyleDuplicate,
                    start,
                    "A component can only have one `<style>` element".to_string(),
                );
            }
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
    // Reference: parser.eat('>', true, false) — required but non-throwing in loose mode
    let closed = parser.eat_required_with_loose(">", false)?;

    // Reference: Loose-mode recovery for unclosed opening tags (element.js lines 365-386)
    if !closed {
        let mut handled = false;
        if let Some(AttributeNode::Attribute(last)) = attributes.last() {
            if last.name == "<" {
                parser.index = last.span.start;
                attributes.pop();
                handled = true;
            }
        }
        if !handled {
            let prev_1 = if parser.index > 0 {
                parser.template.as_bytes().get(parser.index - 1).copied()
            } else {
                None
            };
            let prev_2 = if parser.index > 1 {
                parser.template.as_bytes().get(parser.index - 2).copied()
            } else {
                None
            };
            let current_ch = parser.template.as_bytes().get(parser.index).copied();

            if prev_2 == Some(b'{') && prev_1 == Some(b'/') {
                parser.index -= 2;
            } else if prev_1 == Some(b'{')
                && (current_ch == Some(b'#')
                    || current_ch == Some(b'@')
                    || current_ch == Some(b':'))
            {
                parser.index -= 1;
            } else {
                parser.allow_whitespace();
            }
        }
    }

    let is_void = crate::parser::utils::is_void(&name);

    if self_closing || is_void || !closed {
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
            match_closing_textarea_tag(p)
        });
        eat_closing_textarea_tag(parser);
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
        let data = parser.read_until_closing_tag(&name);
        let content_end = parser.index;
        parser.eat_closing_tag(&name);

        let text_node = FragmentNode::Text(Text {
            span: Span::new(content_start, content_end),
            raw: data,
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
                ErrorKind::ExpectedToken,
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
            // Check if it's a single ExpressionTag (the "expression attribute" case)
            let is_expression_attr = chunks.len() == 1
                && matches!(chunks.first(), Some(AttributeSequenceValue::ExpressionTag(_)));

            if is_expression_attr {
                let chunk = chunks.into_iter().next().unwrap();
                match chunk {
                    AttributeSequenceValue::ExpressionTag(et) => Some(et.expression),
                    _ => unreachable!(),
                }
            } else if name == "svelte:element" {
                // Reference: w.svelte_element_invalid_this — not an expression attribute
                // Fallback: text values become string literals
                if chunks.len() == 1 {
                    let chunk = chunks.into_iter().next().unwrap();
                    match chunk {
                        AttributeSequenceValue::Text(t) => {
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
                        }
                        AttributeSequenceValue::ExpressionTag(et) => Some(et.expression),
                    }
                } else {
                    // Multi-part value: take the first text chunk as literal (buggy Svelte 4 compat)
                    if let Some(first) = chunks.into_iter().next() {
                        match first {
                            AttributeSequenceValue::Text(t) => {
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
                            }
                            AttributeSequenceValue::ExpressionTag(et) => Some(et.expression),
                        }
                    } else {
                        None
                    }
                }
            } else {
                // svelte:component — non-expression value: error
                if !parser.loose {
                    parser.error(
                        ErrorKind::ExpectedExpression,
                        start,
                        format!("`<{name}>` 'this' attribute must be an expression"),
                    );
                }
                None
            }
        }
        AttributeValue::True => {
            // Reference: e.svelte_element_missing_this(definition) for value=true
            if !parser.loose {
                parser.error(
                    ErrorKind::SvelteElementMissingThis,
                    start,
                    format!("`<{name}>` requires a 'this' attribute with a value"),
                );
            }
            None
        }
    }
}

/// Create a FragmentNode directly from name + parts (for self-closing/void/textarea/script/style).
/// Uses name matching to determine element type (reference: element.js lines 157-166).
fn make_element_node<'a>(
    name: &'a str,
    start: usize,
    end: usize,
    name_loc: svelte_ast::span::SourceLocation,
    attributes: Vec<AttributeNode<'a>>,
    fragment: Fragment<'a>,
    this_expression: Option<oxc_ast::ast::Expression<'a>>,
    parser: &Parser<'a>,
) -> FragmentNode<'a> {
    let span = Span::new(start, end);
    match name {
        "svelte:head" => FragmentNode::SvelteHead(SvelteHead {
            span, name, name_loc, attributes, fragment,
        }),
        "svelte:options" => FragmentNode::SvelteOptionsRaw(SvelteOptionsRaw {
            span, name, name_loc, attributes, fragment,
        }),
        "svelte:window" => FragmentNode::SvelteWindow(SvelteWindow {
            span, name, name_loc, attributes, fragment,
        }),
        "svelte:document" => FragmentNode::SvelteDocument(SvelteDocument {
            span, name, name_loc, attributes, fragment,
        }),
        "svelte:body" => FragmentNode::SvelteBody(SvelteBody {
            span, name, name_loc, attributes, fragment,
        }),
        "svelte:element" => {
            let tag = this_expression.unwrap_or_else(|| make_null_literal(parser.allocator));
            FragmentNode::SvelteElement(SvelteElement {
                span, name, name_loc, tag, attributes, fragment,
            })
        }
        "svelte:component" => {
            let expression = this_expression.unwrap_or_else(|| make_null_literal(parser.allocator));
            FragmentNode::SvelteComponent(SvelteComponent {
                span, name, name_loc, expression, attributes, fragment,
            })
        }
        "svelte:self" => FragmentNode::SvelteSelf(SvelteSelf {
            span, name, name_loc, attributes, fragment,
        }),
        "svelte:fragment" => FragmentNode::SvelteFragment(SvelteFragment {
            span, name, name_loc, attributes, fragment,
        }),
        "svelte:boundary" => FragmentNode::SvelteBoundary(SvelteBoundary {
            span, name, name_loc, attributes, fragment,
        }),
        _ if is_component_name(name, parser.loose) => FragmentNode::Component(Component {
            span, name, name_loc, attributes, fragment,
        }),
        "title" if parent_is_head(&parser.stack) => FragmentNode::TitleElement(TitleElement {
            span, name, name_loc, attributes, fragment,
        }),
        "slot" if !parent_is_shadowroot_template(&parser.stack) => {
            FragmentNode::SlotElement(SlotElement {
                span, name, name_loc, attributes, fragment,
            })
        }
        _ => FragmentNode::RegularElement(RegularElement {
            span, name, name_loc, attributes, fragment,
        }),
    }
}

/// Create a StackFrame from name + parts (for elements with children).
/// Uses name matching to determine element type (reference: element.js lines 157-166).
fn make_stack_frame<'a>(
    name: &'a str,
    start: usize,
    name_loc: svelte_ast::span::SourceLocation,
    attributes: Vec<AttributeNode<'a>>,
    this_expression: Option<oxc_ast::ast::Expression<'a>>,
    parser: &Parser<'a>,
) -> StackFrame<'a> {
    match name {
        "svelte:head" => StackFrame::SvelteHead {
            start, name, name_loc, attributes,
        },
        "svelte:options" => StackFrame::SvelteOptions {
            start, name, name_loc, attributes,
        },
        "svelte:window" => StackFrame::SvelteWindow {
            start, name, name_loc, attributes,
        },
        "svelte:document" => StackFrame::SvelteDocument {
            start, name, name_loc, attributes,
        },
        "svelte:body" => StackFrame::SvelteBody {
            start, name, name_loc, attributes,
        },
        "svelte:element" => StackFrame::SvelteElement {
            start, name, name_loc, tag: this_expression, attributes,
        },
        "svelte:component" => StackFrame::SvelteComponent {
            start, name, name_loc, expression: this_expression, attributes,
        },
        "svelte:self" => StackFrame::SvelteSelf {
            start, name, name_loc, attributes,
        },
        "svelte:fragment" => StackFrame::SvelteFragment {
            start, name, name_loc, attributes,
        },
        "svelte:boundary" => StackFrame::SvelteBoundary {
            start, name, name_loc, attributes,
        },
        _ if is_component_name(name, parser.loose) => StackFrame::Component {
            start, name, name_loc, attributes,
        },
        "title" if parent_is_head(&parser.stack) => StackFrame::TitleElement {
            start, name, name_loc, attributes,
        },
        "slot" if !parent_is_shadowroot_template(&parser.stack) => StackFrame::SlotElement {
            start, name, name_loc, attributes,
        },
        _ => StackFrame::RegularElement {
            start, name, name_loc, attributes,
        },
    }
}
