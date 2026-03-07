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

use super::attributes::render_element_attribute_chunks;
use super::expr::{
    call_iife, const_statement, escape_html_expression, join_chunks_expression, string_expr,
    stringify_expression,
};
use super::{render_fragment_expression, render_fragment_nodes_expression, render_node_expression};
use super::scope::{RuntimeScope, is_valid_js_identifier, resolve_expression};

pub(super) fn render_regular_element_expression<'a>(
    ast: AstBuilder<'a>,
    name: &str,
    attributes: &[AttributeNode<'a>],
    children: &'a Fragment<'a>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let textarea_value = if name == "textarea" {
        find_textarea_value_expression(ast, attributes, scope)
    } else {
        None
    };
    let select_value_expression = if name == "select" {
        find_select_value_expression(ast, attributes, scope)
    } else {
        None
    };
    let mut chunks = ast.vec();
    chunks.push(string_expr(ast, &format!("<{name}")));

    let rendered_attributes =
        render_element_attribute_chunks(ast, attributes, scope, Some(name), scope.css_scope());
    for attribute in rendered_attributes {
        chunks.push(attribute);
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
        if let Some(value) = textarea_value {
            chunks.push(escape_html_expression(ast, value));
        } else if let Some(select_value_expression) = &select_value_expression {
            chunks.push(render_select_children_expression(
                ast,
                children,
                scope,
                select_value_expression.clone_in(ast.allocator),
            ));
        } else {
            chunks.push(render_child_fragment_expression(
                ast, name, children, scope,
            ));
        }
        chunks.push(string_expr(ast, &format!("</{name}>")));
    }

    join_chunks_expression(ast, chunks)
}

fn find_textarea_value_expression<'a>(
    ast: AstBuilder<'a>,
    attributes: &[AttributeNode<'a>],
    scope: &RuntimeScope,
) -> Option<Expression<'a>> {
    for attribute in attributes {
        match attribute {
            AttributeNode::Attribute(attribute) if attribute.name == "value" => {
                return Some(attribute_value_expression(ast, &attribute.value, scope));
            }
            AttributeNode::BindDirective(directive) if directive.name == "value" => {
                return Some(resolve_bind_getter_expression(
                    ast,
                    &directive.expression,
                    scope,
                ));
            }
            _ => {}
        }
    }
    None
}

fn find_select_value_expression<'a>(
    ast: AstBuilder<'a>,
    attributes: &[AttributeNode<'a>],
    scope: &RuntimeScope,
) -> Option<Expression<'a>> {
    for attribute in attributes {
        match attribute {
            AttributeNode::Attribute(attribute) if attribute.name == "value" => {
                return Some(attribute_value_expression(ast, &attribute.value, scope));
            }
            AttributeNode::BindDirective(directive) if directive.name == "value" => {
                return Some(resolve_bind_getter_expression(
                    ast,
                    &directive.expression,
                    scope,
                ));
            }
            _ => {}
        }
    }
    None
}

fn render_select_children_expression<'a>(
    ast: AstBuilder<'a>,
    children: &'a Fragment<'a>,
    scope: &RuntimeScope,
    select_value_expression: Expression<'a>,
) -> Expression<'a> {
    let nodes = children
        .nodes
        .iter()
        .filter(|node| !is_whitespace_text_node(node))
        .collect::<Vec<_>>();
    let mut chunks = ast.vec_with_capacity(nodes.len());
    for node in nodes {
        let rendered = match node {
            lux_ast::template::root::FragmentNode::RegularElement(element)
                if element.name == "option" =>
            {
                render_option_element_expression(
                    ast,
                    &element.attributes,
                    &element.fragment,
                    scope,
                    select_value_expression.clone_in(ast.allocator),
                )
            }
            _ => render_node_expression(ast, node, scope),
        };
        chunks.push(rendered);
    }
    join_chunks_expression(ast, chunks)
}

fn render_option_element_expression<'a>(
    ast: AstBuilder<'a>,
    attributes: &[AttributeNode<'a>],
    children: &'a Fragment<'a>,
    scope: &RuntimeScope,
    select_value_expression: Expression<'a>,
) -> Expression<'a> {
    let mut chunks = ast.vec();
    chunks.push(string_expr(ast, "<option"));

    let rendered_attributes =
        render_element_attribute_chunks(ast, attributes, scope, Some("option"), scope.css_scope());
    for attribute in rendered_attributes {
        chunks.push(attribute);
    }

    let option_value_expression = find_option_value_expression(ast, attributes, scope)
        .unwrap_or_else(|| render_fragment_expression(ast, children, scope));
    let selected_expression = ast.expression_binary(
        SPAN,
        option_value_expression,
        BinaryOperator::StrictEquality,
        select_value_expression,
    );
    chunks.push(ast.expression_conditional(
        SPAN,
        selected_expression,
        string_expr(ast, " selected"),
        string_expr(ast, ""),
    ));

    chunks.push(string_expr(ast, ">"));
    chunks.push(render_fragment_expression(ast, children, scope));
    chunks.push(string_expr(ast, "</option>"));
    join_chunks_expression(ast, chunks)
}

fn find_option_value_expression<'a>(
    ast: AstBuilder<'a>,
    attributes: &[AttributeNode<'a>],
    scope: &RuntimeScope,
) -> Option<Expression<'a>> {
    attributes.iter().find_map(|attribute| match attribute {
        AttributeNode::Attribute(attribute) if attribute.name == "value" => {
            Some(attribute_value_expression(ast, &attribute.value, scope))
        }
        _ => None,
    })
}

fn attribute_value_expression<'a>(
    ast: AstBuilder<'a>,
    value: &AttributeValue<'a>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    match value {
        AttributeValue::True => string_expr(ast, ""),
        AttributeValue::ExpressionTag(tag) => {
            resolve_expression(ast, tag.expression.clone_in(ast.allocator), scope)
        }
        AttributeValue::Sequence(chunks) => {
            let mut parts = ast.vec_with_capacity(chunks.len());
            for chunk in chunks {
                let expression = match chunk {
                    TextOrExpressionTag::Text(text) => string_expr(ast, text.raw),
                    TextOrExpressionTag::ExpressionTag(tag) => stringify_expression(
                        ast,
                        resolve_expression(ast, tag.expression.clone_in(ast.allocator), scope),
                    ),
                };
                parts.push(expression);
            }
            join_chunks_expression(ast, parts)
        }
    }
}

fn resolve_bind_getter_expression<'a>(
    ast: AstBuilder<'a>,
    expression: &Expression<'a>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    if let Expression::SequenceExpression(sequence) = expression {
        if let (Some(getter), Some(_setter), None) = (
            sequence.expressions.first(),
            sequence.expressions.get(1),
            sequence.expressions.get(2),
        ) {
            return resolve_expression(ast, getter.clone_in(ast.allocator), scope);
        }
    }
    resolve_expression(ast, expression.clone_in(ast.allocator), scope)
}

fn render_child_fragment_expression<'a>(
    ast: AstBuilder<'a>,
    parent_name: &str,
    children: &'a Fragment<'a>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    if parent_name != "select" {
        return render_fragment_expression(ast, children, scope);
    }

    let nodes = children
        .nodes
        .iter()
        .filter(|node| !is_whitespace_text_node(node))
        .collect::<Vec<_>>();
    render_fragment_nodes_expression(ast, &nodes, scope)
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

fn is_whitespace_text_node(node: &lux_ast::template::root::FragmentNode<'_>) -> bool {
    matches!(node, lux_ast::template::root::FragmentNode::Text(text) if text.raw.trim().is_empty())
}

pub(super) fn render_slot_element_expression<'a>(
    ast: AstBuilder<'a>,
    element: &'a SlotElement<'a>,
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
    element: &'a SvelteElement<'a>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let tag_expression = stringify_expression(
        ast,
        resolve_expression(ast, element.tag.clone_in(ast.allocator), scope),
    );

    let mut statements = ast.vec();
    statements.push(const_statement(ast, "__lux_tag", tag_expression));

    let tag_ident = ast.expression_identifier(SPAN, ast.ident("__lux_tag"));
    let is_void_ident = ast.expression_identifier(SPAN, ast.ident("__lux_is_void"));
    let is_raw_text_ident = ast.expression_identifier(SPAN, ast.ident("__lux_is_raw_text"));
    let mut chunks = ast.vec();
    chunks.push(string_expr(ast, "<!----><"));
    chunks.push(tag_ident.clone_in(ast.allocator));
    let rendered_attributes =
        render_element_attribute_chunks(ast, &element.attributes, scope, None, scope.css_scope());
    for attribute in rendered_attributes {
        chunks.push(attribute);
    }
    chunks.push(string_expr(ast, ">"));

    let children_and_close = join_chunks_expression(
        ast,
        ast.vec_from_array([
            render_fragment_expression(ast, &element.fragment, scope),
            ast.expression_conditional(
                SPAN,
                is_raw_text_ident.clone_in(ast.allocator),
                string_expr(ast, ""),
                string_expr(ast, "<!---->"),
            ),
            string_expr(ast, "</"),
            tag_ident.clone_in(ast.allocator),
            string_expr(ast, ">"),
        ]),
    );
    chunks.push(ast.expression_conditional(
        SPAN,
        is_void_ident.clone_in(ast.allocator),
        string_expr(ast, ""),
        children_and_close,
    ));
    chunks.push(string_expr(ast, "<!---->"));

    statements.push(const_statement(
        ast,
        "__lux_is_void",
        build_tag_name_match_expression(ast, tag_ident.clone_in(ast.allocator), VOID_ELEMENT_NAMES),
    ));
    statements.push(const_statement(
        ast,
        "__lux_is_raw_text",
        build_tag_name_match_expression(
            ast,
            tag_ident.clone_in(ast.allocator),
            RAW_TEXT_ELEMENT_NAMES,
        ),
    ));
    statements.push(ast.statement_return(SPAN, Some(join_chunks_expression(ast, chunks))));
    call_iife(ast, statements)
}

const VOID_ELEMENT_NAMES: &[&str] = &[
    "area",
    "base",
    "br",
    "col",
    "embed",
    "hr",
    "img",
    "input",
    "link",
    "meta",
    "param",
    "source",
    "track",
    "wbr",
];

const RAW_TEXT_ELEMENT_NAMES: &[&str] = &["script", "style", "textarea", "title"];

fn build_tag_name_match_expression<'a>(
    ast: AstBuilder<'a>,
    tag_expression: Expression<'a>,
    names: &[&str],
) -> Expression<'a> {
    let mut iter = names.iter();
    let Some(first) = iter.next() else {
        return ast.expression_boolean_literal(SPAN, false);
    };

    let mut expression = ast.expression_binary(
        SPAN,
        tag_expression.clone_in(ast.allocator),
        BinaryOperator::StrictEquality,
        string_expr(ast, first),
    );
    for name in iter {
        expression = ast.expression_logical(
            SPAN,
            expression,
            LogicalOperator::Or,
            ast.expression_binary(
                SPAN,
                tag_expression.clone_in(ast.allocator),
                BinaryOperator::StrictEquality,
                string_expr(ast, name),
            ),
        );
    }
    expression
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
