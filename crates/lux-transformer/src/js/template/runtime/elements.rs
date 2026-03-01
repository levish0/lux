use lux_ast::template::attribute::{AttributeNode, AttributeValue};
use lux_ast::template::element::{SlotElement, SvelteElement};
use lux_ast::template::root::Fragment;
use lux_ast::template::tag::TextOrExpressionTag;
use lux_utils::elements::{is_load_error_element, is_void};
use oxc_allocator::CloneIn;
use oxc_ast::{
    AstBuilder, NONE,
    ast::{BinaryOperator, Expression, LogicalOperator, PropertyKind},
};
use oxc_span::SPAN;

use super::attributes::render_attribute_expression;
use super::expr::{
    call_iife, const_statement, join_chunks_expression, string_expr, stringify_expression,
};
use super::render_fragment_expression;
use super::scope::{RuntimeScope, is_valid_js_identifier, resolve_expression};

pub(super) fn render_regular_element_expression<'a>(
    ast: AstBuilder<'a>,
    name: &str,
    attributes: &[AttributeNode<'_>],
    children: &Fragment<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let mut chunks = ast.vec();
    chunks.push(string_expr(ast, &format!("<{name}")));

    for attribute in attributes {
        chunks.push(render_attribute_expression(ast, attribute, scope));
    }
    let (capture_onload, capture_onerror) = detect_load_error_captures(name, attributes);
    if capture_onload {
        chunks.push(string_expr(ast, " onload=\"this.__e=event\""));
    }
    if capture_onerror {
        chunks.push(string_expr(ast, " onerror=\"this.__e=event\""));
    }

    chunks.push(string_expr(ast, ">"));
    if !is_void(name) {
        chunks.push(render_fragment_expression(ast, children, scope));
        chunks.push(string_expr(ast, &format!("</{name}>")));
    }

    join_chunks_expression(ast, chunks)
}

fn detect_load_error_captures(name: &str, attributes: &[AttributeNode<'_>]) -> (bool, bool) {
    if !is_load_error_element(name) {
        return (false, false);
    }

    let mut onload = false;
    let mut onerror = false;

    for attribute in attributes {
        match attribute {
            AttributeNode::OnDirective(directive) => match directive.name {
                "load" => onload = true,
                "error" => onerror = true,
                _ => {}
            },
            AttributeNode::Attribute(attribute) => match attribute.name {
                "onload" => onload = true,
                "onerror" => onerror = true,
                _ => {}
            },
            AttributeNode::SpreadAttribute(_) | AttributeNode::UseDirective(_) => {
                onload = true;
                onerror = true;
            }
            _ => {}
        }
    }

    (onload, onerror)
}

pub(super) fn render_slot_element_expression<'a>(
    ast: AstBuilder<'a>,
    element: &SlotElement<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let slot_name = slot_name_expression(ast, &element.attributes, scope);
    let slot_props = build_slot_props_expression(ast, &element.attributes, scope);
    let fallback = render_fragment_expression(ast, &element.fragment, scope);

    let mut statements = ast.vec();
    statements.push(const_statement(ast, "__lux_slot_name", slot_name));
    statements.push(const_statement(
        ast,
        "__lux_slots",
        ast.member_expression_static(
            SPAN,
            ast.expression_identifier(SPAN, ast.ident("_props")),
            ast.identifier_name(SPAN, ast.ident("$$slots")),
            false,
        )
        .into(),
    ));
    let slot_name_ident = ast.expression_identifier(SPAN, ast.ident("__lux_slot_name"));
    let slots_ident = ast.expression_identifier(SPAN, ast.ident("__lux_slots"));
    let named_lookup = ast.expression_logical(
        SPAN,
        slots_ident.clone_in(ast.allocator),
        LogicalOperator::And,
        ast.member_expression_computed(
            SPAN,
            slots_ident.clone_in(ast.allocator),
            slot_name_ident.clone_in(ast.allocator),
            false,
        )
        .into(),
    );
    let default_lookup = ast.expression_logical(
        SPAN,
        named_lookup.clone_in(ast.allocator),
        LogicalOperator::Coalesce,
        ast.member_expression_static(
            SPAN,
            ast.expression_identifier(SPAN, ast.ident("_props")),
            ast.identifier_name(SPAN, ast.ident("children")),
            false,
        )
        .into(),
    );
    let selected_slot_fn = ast.expression_conditional(
        SPAN,
        ast.expression_binary(
            SPAN,
            slot_name_ident.clone_in(ast.allocator),
            BinaryOperator::StrictEquality,
            string_expr(ast, "default"),
        ),
        default_lookup,
        named_lookup,
    );
    statements.push(const_statement(ast, "__lux_slot_fn", selected_slot_fn));
    statements.push(const_statement(ast, "__lux_slot_props", slot_props));

    let slot_fn_ident = ast.expression_identifier(SPAN, ast.ident("__lux_slot_fn"));
    let rendered = ast.expression_conditional(
        SPAN,
        ast.expression_binary(
            SPAN,
            ast.expression_unary(
                SPAN,
                oxc_ast::ast::UnaryOperator::Typeof,
                slot_fn_ident.clone_in(ast.allocator),
            ),
            BinaryOperator::StrictEquality,
            string_expr(ast, "function"),
        ),
        stringify_expression(
            ast,
            ast.expression_call(
                SPAN,
                slot_fn_ident,
                NONE,
                ast.vec1(
                    ast.expression_identifier(SPAN, ast.ident("__lux_slot_props"))
                        .into(),
                ),
                false,
            ),
        ),
        fallback,
    );
    statements.push(ast.statement_return(SPAN, Some(rendered)));
    call_iife(ast, statements)
}

pub(super) fn render_svelte_element_expression<'a>(
    ast: AstBuilder<'a>,
    element: &SvelteElement<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let tag_expression = stringify_expression(
        ast,
        resolve_expression(ast, element.tag.clone_in(ast.allocator), scope),
    );

    let mut statements = ast.vec();
    statements.push(const_statement(ast, "__lux_tag", tag_expression));

    let tag_ident = ast.expression_identifier(SPAN, ast.ident("__lux_tag"));
    let mut chunks = ast.vec();
    chunks.push(string_expr(ast, "<"));
    chunks.push(tag_ident.clone_in(ast.allocator));
    for attribute in &element.attributes {
        chunks.push(render_attribute_expression(ast, attribute, scope));
    }
    chunks.push(string_expr(ast, ">"));
    chunks.push(render_fragment_expression(ast, &element.fragment, scope));
    chunks.push(string_expr(ast, "</"));
    chunks.push(tag_ident);
    chunks.push(string_expr(ast, ">"));

    statements.push(ast.statement_return(SPAN, Some(join_chunks_expression(ast, chunks))));
    call_iife(ast, statements)
}

fn slot_name_expression<'a>(
    ast: AstBuilder<'a>,
    attributes: &[AttributeNode<'_>],
    scope: &RuntimeScope,
) -> Expression<'a> {
    for attribute in attributes {
        if let AttributeNode::Attribute(attribute) = attribute {
            if attribute.name == "name" {
                return attribute_value_to_component_prop_expression(ast, &attribute.value, scope);
            }
        }
    }
    string_expr(ast, "default")
}

fn build_slot_props_expression<'a>(
    ast: AstBuilder<'a>,
    attributes: &[AttributeNode<'_>],
    scope: &RuntimeScope,
) -> Expression<'a> {
    let mut properties = ast.vec();

    for attribute in attributes {
        match attribute {
            AttributeNode::Attribute(attribute) => {
                if attribute.name != "name" {
                    properties.push(object_init_property(
                        ast,
                        attribute.name,
                        attribute_value_to_component_prop_expression(ast, &attribute.value, scope),
                    ));
                }
            }
            AttributeNode::SpreadAttribute(attribute) => {
                let expression =
                    resolve_expression(ast, attribute.expression.clone_in(ast.allocator), scope);
                properties.push(ast.object_property_kind_spread_property(SPAN, expression));
            }
            AttributeNode::BindDirective(attribute) => {
                if attribute.name == "this" {
                    continue;
                }
                let expression =
                    resolve_expression(ast, attribute.expression.clone_in(ast.allocator), scope);
                properties.push(object_init_property(ast, attribute.name, expression));
            }
            AttributeNode::ClassDirective(_)
            | AttributeNode::StyleDirective(_)
            | AttributeNode::OnDirective(_)
            | AttributeNode::TransitionDirective(_)
            | AttributeNode::AnimateDirective(_)
            | AttributeNode::UseDirective(_)
            | AttributeNode::LetDirective(_)
            | AttributeNode::AttachTag(_) => {}
        }
    }

    ast.expression_object(SPAN, properties)
}

pub(super) fn attribute_value_to_component_prop_expression<'a>(
    ast: AstBuilder<'a>,
    value: &AttributeValue<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    match value {
        AttributeValue::True => ast.expression_boolean_literal(SPAN, true),
        AttributeValue::ExpressionTag(tag) => {
            resolve_expression(ast, tag.expression.clone_in(ast.allocator), scope)
        }
        AttributeValue::Sequence(chunks) => {
            let mut parts = ast.vec();
            for chunk in chunks {
                let chunk_expression = match chunk {
                    TextOrExpressionTag::Text(text) => string_expr(ast, text.raw),
                    TextOrExpressionTag::ExpressionTag(tag) => stringify_expression(
                        ast,
                        resolve_expression(ast, tag.expression.clone_in(ast.allocator), scope),
                    ),
                };
                parts.push(chunk_expression);
            }
            join_chunks_expression(ast, parts)
        }
    }
}

pub(super) fn object_init_property<'a>(
    ast: AstBuilder<'a>,
    name: &str,
    value: Expression<'a>,
) -> oxc_ast::ast::ObjectPropertyKind<'a> {
    let (key, computed) = if is_valid_js_identifier(name) {
        (
            ast.property_key_static_identifier(SPAN, ast.ident(name)),
            false,
        )
    } else {
        (string_expr(ast, name).into(), false)
    };

    ast.object_property_kind_object_property(
        SPAN,
        PropertyKind::Init,
        key,
        value,
        false,
        false,
        computed,
    )
}
