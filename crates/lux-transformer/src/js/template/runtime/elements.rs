use lux_ast::template::attribute::{AttributeNode, AttributeValue};
use lux_ast::template::element::{Component, SlotElement, SvelteComponent, SvelteElement, SvelteSelf};
use lux_ast::template::root::{Fragment, FragmentNode};
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
use super::{render_fragment_expression, render_fragment_nodes_expression};
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
                ast.vec1(ast.expression_identifier(SPAN, ast.ident("__lux_slot_props")).into()),
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

pub(super) fn render_svelte_self_expression<'a>(
    ast: AstBuilder<'a>,
    component: &SvelteSelf<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let callee = ast.member_expression_static(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident("_props")),
        ast.identifier_name(SPAN, ast.ident("__lux_self")),
        false,
    );
    render_component_like_expression(
        ast,
        callee.into(),
        &component.attributes,
        &component.fragment,
        scope,
    )
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

    let has_children_prop = component_has_children_prop(attributes);
    let slot_groups = collect_component_slot_groups(fragment);
    if !slot_groups.is_empty() {
        let mut slot_properties = ast.vec();

        for (slot_name, nodes) in slot_groups {
            let slot_function = build_slot_function_for_nodes(ast, &nodes, scope);

            if slot_name == "default" && !has_children_prop {
                properties.push(object_init_property(
                    ast,
                    "children",
                    slot_function.clone_in(ast.allocator),
                ));
            }

            slot_properties.push(object_init_property(ast, slot_name, slot_function));
        }

        properties.push(object_init_property(
            ast,
            "$$slots",
            ast.expression_object(SPAN, slot_properties),
        ));
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

fn build_slot_function_for_nodes<'a>(
    ast: AstBuilder<'a>,
    nodes: &[&FragmentNode<'_>],
    scope: &RuntimeScope,
) -> Expression<'a> {
    let child_expression = render_fragment_nodes_expression(ast, nodes, scope);
    let params =
        ast.alloc_formal_parameters(SPAN, FormalParameterKind::FormalParameter, ast.vec(), NONE);
    let body = ast.alloc_function_body(
        SPAN,
        ast.vec(),
        ast.vec1(ast.statement_return(SPAN, Some(child_expression))),
    );
    ast.expression_function(
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
    )
}

fn collect_component_slot_groups<'n>(fragment: &'n Fragment<'n>) -> Vec<(&'n str, Vec<&'n FragmentNode<'n>>)> {
    let mut groups: Vec<(&'n str, Vec<&'n FragmentNode<'n>>)> = Vec::new();

    for node in &fragment.nodes {
        let slot_name = component_child_slot_name(node).unwrap_or("default");
        if let Some((_, nodes)) = groups
            .iter_mut()
            .find(|(name, _)| *name == slot_name)
        {
            nodes.push(node);
        } else {
            groups.push((slot_name, vec![node]));
        }
    }

    groups
}

fn component_child_slot_name<'a>(node: &'a FragmentNode<'a>) -> Option<&'a str> {
    let attributes = match node {
        FragmentNode::RegularElement(element) => Some(element.attributes.as_slice()),
        FragmentNode::Component(element) => Some(element.attributes.as_slice()),
        FragmentNode::SvelteComponent(element) => Some(element.attributes.as_slice()),
        FragmentNode::SvelteSelf(element) => Some(element.attributes.as_slice()),
        FragmentNode::SvelteElement(element) => Some(element.attributes.as_slice()),
        FragmentNode::SvelteFragment(element) => Some(element.attributes.as_slice()),
        FragmentNode::SvelteHead(element) => Some(element.attributes.as_slice()),
        FragmentNode::SvelteBody(element) => Some(element.attributes.as_slice()),
        FragmentNode::SvelteWindow(element) => Some(element.attributes.as_slice()),
        FragmentNode::SvelteDocument(element) => Some(element.attributes.as_slice()),
        FragmentNode::SvelteBoundary(element) => Some(element.attributes.as_slice()),
        FragmentNode::SlotElement(element) => Some(element.attributes.as_slice()),
        FragmentNode::TitleElement(element) => Some(element.attributes.as_slice()),
        FragmentNode::SvelteOptionsRaw(element) => Some(element.attributes.as_slice()),
        FragmentNode::Text(_)
        | FragmentNode::ExpressionTag(_)
        | FragmentNode::HtmlTag(_)
        | FragmentNode::ConstTag(_)
        | FragmentNode::DebugTag(_)
        | FragmentNode::RenderTag(_)
        | FragmentNode::AttachTag(_)
        | FragmentNode::Comment(_)
        | FragmentNode::IfBlock(_)
        | FragmentNode::EachBlock(_)
        | FragmentNode::AwaitBlock(_)
        | FragmentNode::KeyBlock(_)
        | FragmentNode::SnippetBlock(_) => None,
    }?;

    for attribute in attributes {
        if let AttributeNode::Attribute(attribute) = attribute {
            if attribute.name != "slot" {
                continue;
            }
            if let AttributeValue::Sequence(chunks) = &attribute.value {
                if chunks.len() == 1 {
                    if let TextOrExpressionTag::Text(text) = &chunks[0] {
                        return Some(text.data);
                    }
                }
            }
        }
    }

    None
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
