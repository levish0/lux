use lux_ast::template::attribute::{AttributeNode, AttributeValue};
use lux_ast::template::element::{Component, SvelteComponent, SvelteElement};
use lux_ast::template::root::Fragment;
use lux_ast::template::tag::TextOrExpressionTag;
use lux_utils::elements::is_void;
use oxc_allocator::CloneIn;
use oxc_ast::{
    AstBuilder, NONE,
    ast::{BinaryOperator, Expression, FormalParameterKind, FunctionType, LogicalOperator, PropertyKind},
};
use oxc_span::SPAN;

use super::attributes::render_attribute_expression;
use super::expr::{call_iife, concat_expr, const_statement, string_expr, stringify_expression};
use super::render_fragment_expression;
use super::scope::{RuntimeScope, is_valid_js_identifier, resolve_expression};

pub(super) fn render_regular_element_expression<'a>(
    ast: AstBuilder<'a>,
    name: &str,
    attributes: &[AttributeNode<'_>],
    children: &Fragment<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let mut out = string_expr(ast, &format!("<{name}"));

    for attribute in attributes {
        out = concat_expr(ast, out, render_attribute_expression(ast, attribute, scope));
    }

    out = concat_expr(ast, out, string_expr(ast, ">"));
    if !is_void(name) {
        out = concat_expr(ast, out, render_fragment_expression(ast, children, scope));
        out = concat_expr(ast, out, string_expr(ast, &format!("</{name}>")));
    }

    out
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
    let mut out = concat_expr(ast, string_expr(ast, "<"), tag_ident.clone_in(ast.allocator));
    for attribute in &element.attributes {
        out = concat_expr(ast, out, render_attribute_expression(ast, attribute, scope));
    }
    out = concat_expr(ast, out, string_expr(ast, ">"));
    out = concat_expr(ast, out, render_fragment_expression(ast, &element.fragment, scope));
    out = concat_expr(ast, out, string_expr(ast, "</"));
    out = concat_expr(ast, out, tag_ident);
    out = concat_expr(ast, out, string_expr(ast, ">"));

    statements.push(ast.statement_return(SPAN, Some(out)));
    call_iife(ast, statements)
}

pub(super) fn render_component_expression<'a>(
    ast: AstBuilder<'a>,
    component: &Component<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let callee = resolve_expression(
        ast,
        ast.expression_identifier(SPAN, ast.ident(component.name)),
        scope,
    );
    render_component_like_expression(ast, callee, &component.attributes, &component.fragment, scope)
}

pub(super) fn render_svelte_component_expression<'a>(
    ast: AstBuilder<'a>,
    component: &SvelteComponent<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let callee = resolve_expression(ast, component.expression.clone_in(ast.allocator), scope);
    render_component_like_expression(ast, callee, &component.attributes, &component.fragment, scope)
}

fn render_component_like_expression<'a>(
    ast: AstBuilder<'a>,
    callee: Expression<'a>,
    attributes: &[AttributeNode<'_>],
    fragment: &Fragment<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let props_expression = build_component_props_expression(ast, attributes, fragment, scope);

    let component_ident = ast.expression_identifier(SPAN, ast.ident("__lux_component"));
    let props_ident = ast.expression_identifier(SPAN, ast.ident("__lux_component_props"));
    let render_member = ast.member_expression_static(
        SPAN,
        component_ident.clone_in(ast.allocator),
        ast.identifier_name(SPAN, ast.ident("render")),
        false,
    );
    let has_render = ast.expression_logical(
        SPAN,
        component_ident.clone_in(ast.allocator),
        LogicalOperator::And,
        ast.expression_binary(
            SPAN,
            ast.expression_unary(SPAN, oxc_ast::ast::UnaryOperator::Typeof, render_member.clone_in(ast.allocator).into()),
            BinaryOperator::StrictEquality,
            string_expr(ast, "function"),
        ),
    );
    let render_call = ast.expression_call(
        SPAN,
        render_member.into(),
        NONE,
        ast.vec1(props_ident.clone_in(ast.allocator).into()),
        false,
    );
    let function_call = ast.expression_call(
        SPAN,
        component_ident.clone_in(ast.allocator),
        NONE,
        ast.vec1(props_ident.clone_in(ast.allocator).into()),
        false,
    );
    let is_callable = ast.expression_binary(
        SPAN,
        ast.expression_unary(
            SPAN,
            oxc_ast::ast::UnaryOperator::Typeof,
            component_ident.clone_in(ast.allocator),
        ),
        BinaryOperator::StrictEquality,
        string_expr(ast, "function"),
    );
    let rendered = ast.expression_conditional(
        SPAN,
        has_render,
        render_call,
        ast.expression_conditional(SPAN, is_callable, function_call, string_expr(ast, "")),
    );

    let statements = ast.vec_from_array([
        const_statement(ast, "__lux_component", callee),
        const_statement(ast, "__lux_component_props", props_expression),
        ast.statement_return(SPAN, Some(rendered)),
    ]);
    stringify_expression(ast, call_iife(ast, statements))
}

fn build_component_props_expression<'a>(
    ast: AstBuilder<'a>,
    attributes: &[AttributeNode<'_>],
    fragment: &Fragment<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let mut properties = ast.vec();

    for attribute in attributes {
        match attribute {
            AttributeNode::Attribute(attribute) => {
                properties.push(object_init_property(
                    ast,
                    attribute.name,
                    attribute_value_to_component_prop_expression(ast, &attribute.value, scope),
                ));
            }
            AttributeNode::SpreadAttribute(attribute) => {
                let expression =
                    resolve_expression(ast, attribute.expression.clone_in(ast.allocator), scope);
                properties.push(ast.object_property_kind_spread_property(SPAN, expression));
            }
            AttributeNode::BindDirective(attribute) => {
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

    if !fragment.nodes.is_empty() && !component_has_children_prop(attributes) {
        let child_expression = render_fragment_expression(ast, fragment, scope);
        let params =
            ast.alloc_formal_parameters(SPAN, FormalParameterKind::FormalParameter, ast.vec(), NONE);
        let body = ast.alloc_function_body(
            SPAN,
            ast.vec(),
            ast.vec1(ast.statement_return(SPAN, Some(child_expression))),
        );
        let child_function = ast.expression_function(
            SPAN,
            FunctionType::FunctionExpression,
            None,
            false,
            false,
            false,
            NONE,
            NONE,
            params,
            NONE,
            Some(body),
        );
        properties.push(object_init_property(ast, "children", child_function));
    }

    ast.expression_object(SPAN, properties)
}

fn component_has_children_prop(attributes: &[AttributeNode<'_>]) -> bool {
    attributes.iter().any(|attribute| match attribute {
        AttributeNode::Attribute(attribute) => attribute.name == "children",
        AttributeNode::BindDirective(attribute) => attribute.name == "children",
        _ => false,
    })
}

fn attribute_value_to_component_prop_expression<'a>(
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
            let mut out = string_expr(ast, "");
            for chunk in chunks {
                let chunk_expression = match chunk {
                    TextOrExpressionTag::Text(text) => string_expr(ast, text.raw),
                    TextOrExpressionTag::ExpressionTag(tag) => stringify_expression(
                        ast,
                        resolve_expression(ast, tag.expression.clone_in(ast.allocator), scope),
                    ),
                };
                out = concat_expr(ast, out, chunk_expression);
            }
            out
        }
    }
}

fn object_init_property<'a>(
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
