use svelte_ast::elements::{Component, RegularElement};
use svelte_ast::node::FragmentNode;
use svelte_ast::root::Fragment;
use svelte_ast::span::Span;
use svelte_ast::text::Comment;

use crate::parser::{ParseError, Parser, StackFrame};

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
    let data_start = parser.index;
    loop {
        if parser.index >= parser.template.len() {
            break;
        }
        if parser.match_str("-->") {
            let data = &parser.template[data_start..parser.index];
            parser.index += 3;
            parser.append(FragmentNode::Comment(Comment {
                span: Span::new(start, parser.index),
                data: data.to_string(),
            }));
            return;
        }
        parser.index += 1;
    }
    let data = &parser.template[data_start..parser.index];
    parser.append(FragmentNode::Comment(Comment {
        span: Span::new(start, parser.index),
        data: data.to_string(),
    }));
}

/// Handle a closing tag: `</name>`
fn close_tag(parser: &mut Parser) -> Result<(), ParseError> {
    parser.allow_whitespace();
    let (name, _name_start, _name_end) = parser.read_identifier();
    let name = name.to_string();
    parser.allow_whitespace();
    parser.eat_required(">")?;

    // Find matching open element on the stack
    let mut found = false;
    for frame in parser.stack.iter().rev() {
        match frame {
            StackFrame::RegularElement { name: n, .. } | StackFrame::Component { name: n, .. } => {
                if *n == name {
                    found = true;
                    break;
                }
            }
            _ => {}
        }
    }

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

        match frame {
            StackFrame::RegularElement {
                start,
                name: n,
                name_loc,
                attributes,
            } => {
                let is_match = n == name;
                let node = FragmentNode::RegularElement(RegularElement {
                    span: Span::new(start, parser.index),
                    name_loc,
                    name: n,
                    attributes,
                    fragment: Fragment {
                        nodes: fragment_nodes,
                    },
                });
                parser.append(node);
                if is_match {
                    break;
                }
            }
            StackFrame::Component {
                start,
                name: n,
                name_loc,
                attributes,
            } => {
                let is_match = n == name;
                let node = FragmentNode::Component(Component {
                    span: Span::new(start, parser.index),
                    name_loc,
                    name: n,
                    attributes,
                    fragment: Fragment {
                        nodes: fragment_nodes,
                    },
                });
                parser.append(node);
                if is_match {
                    break;
                }
            }
            _ => {
                // Non-element frame — shouldn't happen in well-formed input
            }
        }
    }

    Ok(())
}

/// Handle an opening tag: `<name ...>`
fn open_tag(parser: &mut Parser, start: usize) -> Result<(), ParseError> {
    let (name, name_start, name_end) = parser.read_identifier();
    if name.is_empty() {
        // Not a valid tag — treat as text
        parser.index = start;
        super::text::text(parser);
        return Ok(());
    }
    let name = name.to_string();
    let name_loc = parser.source_location(name_start, name_end);

    let attributes = read_attributes(parser);

    parser.allow_whitespace();

    let self_closing = parser.eat("/");
    parser.eat_required(">")?;

    let is_component =
        name.chars().next().map_or(false, |c| c.is_uppercase()) || name.contains('.');

    let is_void = crate::parser::utils::is_void(&name);

    if self_closing || is_void {
        if is_component {
            parser.append(FragmentNode::Component(Component {
                span: Span::new(start, parser.index),
                name_loc,
                name,
                attributes,
                fragment: Fragment { nodes: Vec::new() },
            }));
        } else {
            parser.append(FragmentNode::RegularElement(RegularElement {
                span: Span::new(start, parser.index),
                name_loc,
                name,
                attributes,
                fragment: Fragment { nodes: Vec::new() },
            }));
        }
    } else {
        if is_component {
            parser.stack.push(StackFrame::Component {
                start,
                name,
                name_loc,
                attributes,
            });
        } else {
            parser.stack.push(StackFrame::RegularElement {
                start,
                name,
                name_loc,
                attributes,
            });
        }
        parser.fragments.push(Vec::new());
    }

    Ok(())
}

/// Read attributes until `>` or `/>`.
/// TODO: implement proper attribute parsing with directives, expressions, etc.
fn read_attributes<'a>(parser: &mut Parser<'a>) -> Vec<svelte_ast::node::AttributeNode<'a>> {
    let mut attributes = Vec::new();

    loop {
        parser.allow_whitespace();

        if parser.index >= parser.template.len() {
            break;
        }

        if parser.match_str(">") || parser.match_str("/>") {
            break;
        }

        // Spread: {...expr}
        if parser.match_str("{") {
            skip_attribute_expression(parser);
            continue;
        }

        // Regular attribute
        let (attr_name, attr_start, _attr_end) = parser.read_identifier();
        if attr_name.is_empty() {
            parser.index += 1;
            continue;
        }
        let attr_name = attr_name.to_string();

        parser.allow_whitespace();

        let value_end;
        if parser.eat("=") {
            parser.allow_whitespace();
            if parser.match_str("\"") || parser.match_str("'") {
                skip_quoted_value(parser);
            } else if parser.match_str("{") {
                skip_attribute_expression(parser);
            } else {
                while parser.index < parser.template.len() {
                    let ch = parser.template.as_bytes()[parser.index];
                    if ch == b' '
                        || ch == b'\t'
                        || ch == b'\r'
                        || ch == b'\n'
                        || ch == b'>'
                        || ch == b'/'
                    {
                        break;
                    }
                    parser.index += 1;
                }
            }
            value_end = parser.index;
        } else {
            value_end = parser.index;
        }

        attributes.push(svelte_ast::node::AttributeNode::Attribute(
            svelte_ast::attributes::Attribute {
                span: Span::new(attr_start, value_end),
                name: attr_name,
                name_loc: None,
                value: svelte_ast::attributes::AttributeValue::True,
            },
        ));
    }

    attributes
}

/// Skip an attribute expression `{...}`.
fn skip_attribute_expression(parser: &mut Parser) {
    if !parser.eat("{") {
        return;
    }
    let mut depth = 1u32;
    while parser.index < parser.template.len() && depth > 0 {
        let ch = parser.template.as_bytes()[parser.index];
        match ch {
            b'{' => depth += 1,
            b'}' => depth -= 1,
            b'\'' | b'"' | b'`' => {
                skip_string(parser, ch);
                continue;
            }
            _ => {}
        }
        if depth > 0 {
            parser.index += 1;
        }
    }
    if depth == 0 {
        parser.index += 1;
    }
}

/// Skip a quoted attribute value `"..."` or `'...'`.
fn skip_quoted_value(parser: &mut Parser) {
    let quote = parser.template.as_bytes()[parser.index];
    parser.index += 1;
    while parser.index < parser.template.len() {
        let ch = parser.template.as_bytes()[parser.index];
        if ch == quote {
            parser.index += 1;
            return;
        }
        if ch == b'{' {
            skip_attribute_expression(parser);
            continue;
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
