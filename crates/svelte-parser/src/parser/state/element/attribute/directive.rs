use crate::error::ErrorKind;
use crate::parser::Parser;
use svelte_ast::attributes::{
    AnimateDirective, Attribute, AttributeSequenceValue, AttributeValue, BindDirective,
    ClassDirective, LetDirective, OnDirective, StyleDirective, TransitionDirective, UseDirective,
};
use svelte_ast::node::AttributeNode;
use svelte_ast::span::{SourceLocation, Span};

/// Build a directive node from parsed attribute parts.
/// Reference: element.js lines 619-693
pub fn build_directive<'a>(
    parser: &mut Parser<'a>,
    prefix: &str,
    dir_name: &'a str,
    modifiers: Vec<&'a str>,
    value: AttributeValue<'a>,
    name_loc: SourceLocation,
    start: usize,
    end: usize,
) -> Option<AttributeNode<'a>> {
    let span = Span::new(start, end);

    // Reference: if (directive_name === '') e.directive_missing_name(...)
    if dir_name.is_empty() {
        if !parser.loose {
            parser.error(
                ErrorKind::DirectiveMissingName,
                start,
                format!("`{}:` directive is missing a name", prefix),
            );
        }
        return None;
    }

    // StyleDirective gets the full value (can be true, ExpressionTag, or Sequence)
    if prefix == "style" {
        return Some(AttributeNode::StyleDirective(StyleDirective {
            span,
            name: dir_name,
            name_loc: Some(name_loc),
            value,
            modifiers,
        }));
    }

    // For other directives, extract expression from value
    // Reference: element.js lines 641-656
    let expression = match value {
        AttributeValue::True => None,
        AttributeValue::ExpressionTag(et) => Some(et.expression),
        AttributeValue::Sequence(chunks) => {
            // Reference: attribute_contains_text = value.length > 1 || first_value.type === 'Text'
            let contains_text =
                chunks.len() > 1 || matches!(chunks.first(), Some(AttributeSequenceValue::Text(_)));
            if contains_text {
                // Reference: e.directive_invalid_value(first_value.start)
                if !parser.loose {
                    let err_pos = match chunks.first() {
                        Some(AttributeSequenceValue::Text(t)) => t.span.start,
                        Some(AttributeSequenceValue::ExpressionTag(et)) => et.span.start,
                        None => start,
                    };
                    parser.error(
                        ErrorKind::DirectiveInvalidValue,
                        err_pos,
                        format!(
                            "`{}:{}` directive value must be a single expression",
                            prefix, dir_name
                        ),
                    );
                }
                None
            } else if chunks.len() == 1 {
                let chunk = chunks.into_iter().next().unwrap();
                match chunk {
                    AttributeSequenceValue::ExpressionTag(et) => Some(et.expression),
                    _ => None,
                }
            } else {
                None
            }
        }
    };

    // If no expression but it's bind or class, create implicit identifier
    let expression = expression.or_else(|| {
        if prefix == "bind" || prefix == "class" {
            let id_start = start + prefix.len() + 1;
            let id_end = id_start + dir_name.len();
            Some(make_identifier(parser, dir_name, id_start, id_end))
        } else {
            None
        }
    });

    match prefix {
        "on" => Some(AttributeNode::OnDirective(OnDirective {
            span,
            name: dir_name,
            name_loc: Some(name_loc),
            expression,
            modifiers,
        })),
        "bind" => Some(AttributeNode::BindDirective(BindDirective {
            span,
            name: dir_name,
            name_loc: Some(name_loc),
            expression: expression.unwrap_or_else(|| make_identifier(parser, dir_name, start, end)),
            modifiers,
        })),
        "class" => Some(AttributeNode::ClassDirective(ClassDirective {
            span,
            name: dir_name,
            name_loc: Some(name_loc),
            expression: expression.unwrap_or_else(|| make_identifier(parser, dir_name, start, end)),
            modifiers,
        })),
        "use" => Some(AttributeNode::UseDirective(UseDirective {
            span,
            name: dir_name,
            name_loc: Some(name_loc),
            expression,
            modifiers,
        })),
        "animate" => Some(AttributeNode::AnimateDirective(AnimateDirective {
            span,
            name: dir_name,
            name_loc: Some(name_loc),
            expression,
            modifiers,
        })),
        "transition" | "in" | "out" => {
            Some(AttributeNode::TransitionDirective(TransitionDirective {
                span,
                name: dir_name,
                name_loc: Some(name_loc),
                expression,
                modifiers,
                intro: prefix == "in" || prefix == "transition",
                outro: prefix == "out" || prefix == "transition",
            }))
        }
        "let" => Some(AttributeNode::LetDirective(LetDirective {
            span,
            name: dir_name,
            name_loc: Some(name_loc),
            expression,
            modifiers,
        })),
        _ => {
            // For unknown prefix, create attribute with combined name
            // This requires allocating a new string in the arena
            let combined = parser
                .allocator
                .alloc_str(&format!("{}:{}", prefix, dir_name));
            Some(AttributeNode::Attribute(Attribute {
                span,
                name: combined,
                name_loc: Some(name_loc),
                value: AttributeValue::True,
            }))
        }
    }
}

/// Create an Identifier expression for implicit directive names.
pub fn make_identifier<'a>(
    parser: &Parser<'a>,
    name: &str,
    start: usize,
    end: usize,
) -> oxc_ast::ast::Expression<'a> {
    use std::cell::Cell;
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

/// Check if a prefix is a valid directive prefix.
/// Reference: get_directive_type in element.js
pub fn get_directive_type(prefix: &str) -> bool {
    matches!(
        prefix,
        "on" | "bind" | "class" | "style" | "use" | "animate" | "transition" | "in" | "out" | "let"
    )
}
