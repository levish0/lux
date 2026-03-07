use lux_ast::template::attribute::{AttributeNode, AttributeValue};
use lux_ast::template::directive::{EventModifier, LetDirective};
use lux_ast::template::element::{Component, SvelteComponent, SvelteSelf};
use lux_ast::template::root::{Fragment, FragmentNode};
use lux_ast::template::tag::TextOrExpressionTag;
use oxc_allocator::CloneIn;
use oxc_ast::{
    AstBuilder, NONE,
    ast::{
        AssignmentOperator, BinaryOperator, Expression, FormalParameterKind, FunctionType,
        LogicalOperator, PropertyKind, Statement, VariableDeclarationKind,
    },
};
use oxc_span::SPAN;

use super::elements::{attribute_value_to_component_prop_expression, object_init_property};
use super::expr::{call_iife, const_statement, string_expr, stringify_expression};
use super::render_fragment_nodes_expression;
use super::scope::{RuntimeScope, resolve_expression};

pub(super) fn render_component_expression<'a>(
    ast: AstBuilder<'a>,
    component: &Component<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let callee = resolve_expression(ast, component_name_expression(ast, component.name), scope);
    render_component_like_expression(
        ast,
        callee,
        &component.attributes,
        &component.fragment,
        scope,
    )
}

fn component_name_expression<'a>(ast: AstBuilder<'a>, name: &str) -> Expression<'a> {
    let mut segments = name.split('.');
    let Some(first) = segments.next() else {
        return string_expr(ast, "");
    };

    let mut expression = ast.expression_identifier(SPAN, ast.ident(first));
    for segment in segments {
        expression = ast
            .member_expression_static(
                SPAN,
                expression,
                ast.identifier_name(SPAN, ast.ident(segment)),
                false,
            )
            .into();
    }

    expression
}

pub(super) fn render_svelte_component_expression<'a>(
    ast: AstBuilder<'a>,
    component: &SvelteComponent<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let callee = resolve_expression(ast, component.expression.clone_in(ast.allocator), scope);
    render_component_like_expression(
        ast,
        callee,
        &component.attributes,
        &component.fragment,
        scope,
    )
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
    let (props_expression, bind_this_expression) =
        build_component_props_expression(ast, attributes, fragment, scope);

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
            ast.expression_unary(
                SPAN,
                oxc_ast::ast::UnaryOperator::Typeof,
                render_member.clone_in(ast.allocator).into(),
            ),
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

    let mut statements = ast.vec_with_capacity(4);
    statements.push(const_statement(ast, "__lux_component", callee));
    statements.push(const_statement(
        ast,
        "__lux_component_props",
        props_expression,
    ));
    if let Some(bind_expression) = bind_this_expression {
        if let Some(bind_statement) =
            build_component_bind_this_statement(ast, &bind_expression, scope)
        {
            statements.push(bind_statement);
        }
    }
    statements.push(ast.statement_return(SPAN, Some(rendered)));
    stringify_expression(ast, call_iife(ast, statements))
}

fn build_component_props_expression<'a>(
    ast: AstBuilder<'a>,
    attributes: &[AttributeNode<'_>],
    fragment: &Fragment<'_>,
    scope: &RuntimeScope,
) -> (Expression<'a>, Option<Expression<'a>>) {
    let mut properties = ast.vec();
    let mut event_handlers: Vec<(&str, Vec<Expression<'a>>)> = Vec::new();
    let mut bind_this_expression: Option<Expression<'a>> = None;

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
                if attribute.name == "this" {
                    bind_this_expression = Some(attribute.expression.clone_in(ast.allocator));
                    continue;
                }
                push_component_bind_properties(
                    ast,
                    &mut properties,
                    attribute.name,
                    attribute.expression.clone_in(ast.allocator),
                    scope,
                );
            }
            AttributeNode::OnDirective(attribute) => {
                let mut resolved = if let Some(expression) = &attribute.expression {
                    resolve_expression(ast, expression.clone_in(ast.allocator), scope)
                } else {
                    build_component_event_forward_expression(ast, attribute.name)
                };
                if attribute.modifiers.contains(&EventModifier::Once) {
                    resolved = ast.expression_call(
                        SPAN,
                        ast.expression_identifier(SPAN, ast.ident("__lux_once")),
                        NONE,
                        ast.vec1(resolved.into()),
                        false,
                    );
                }
                if let Some((_, handlers)) = event_handlers
                    .iter_mut()
                    .find(|(name, _)| *name == attribute.name)
                {
                    handlers.push(resolved);
                } else {
                    event_handlers.push((attribute.name, vec![resolved]));
                }
            }
            AttributeNode::ClassDirective(_)
            | AttributeNode::StyleDirective(_)
            | AttributeNode::TransitionDirective(_)
            | AttributeNode::AnimateDirective(_)
            | AttributeNode::UseDirective(_)
            | AttributeNode::LetDirective(_)
            | AttributeNode::AttachTag(_) => {}
        }
    }

    if !event_handlers.is_empty() {
        let mut events_properties = ast.vec();
        for (name, handlers) in event_handlers {
            let value = if handlers.len() == 1 {
                handlers
                    .into_iter()
                    .next()
                    .expect("single handler must exist")
            } else {
                let mut items = ast.vec();
                for handler in handlers {
                    items.push(handler.into());
                }
                ast.expression_array(SPAN, items)
            };
            events_properties.push(object_init_property(ast, name, value));
        }
        properties.push(object_init_property(
            ast,
            "$$events",
            ast.expression_object(SPAN, events_properties),
        ));
    }

    let has_children_prop = component_has_children_prop(attributes);
    let default_slot_let_bindings = collect_default_slot_let_bindings(attributes);
    let slot_groups = collect_component_slot_groups(fragment);
    if !slot_groups.is_empty() {
        let mut slot_properties = ast.vec();

        for group in slot_groups {
            let mut let_bindings = group.let_bindings;
            if group.slot_name == "default" {
                for binding in &default_slot_let_bindings {
                    if !let_bindings
                        .iter()
                        .any(|existing| existing.local_name == binding.local_name)
                    {
                        let_bindings.push(*binding);
                    }
                }
            }
            let slot_function =
                build_slot_function_for_nodes(ast, &group.nodes, &let_bindings, scope);

            if group.slot_name == "default" && !has_children_prop && let_bindings.is_empty() {
                properties.push(object_init_property(
                    ast,
                    "children",
                    slot_function.clone_in(ast.allocator),
                ));
            }

            slot_properties.push(object_init_property(ast, group.slot_name, slot_function));
        }

        properties.push(object_init_property(
            ast,
            "$$slots",
            ast.expression_object(SPAN, slot_properties),
        ));
    }

    (
        ast.expression_object(SPAN, properties),
        bind_this_expression,
    )
}

fn push_component_bind_properties<'a>(
    ast: AstBuilder<'a>,
    properties: &mut oxc_allocator::Vec<'a, oxc_ast::ast::ObjectPropertyKind<'a>>,
    bind_name: &str,
    bind_expression: Expression<'a>,
    scope: &RuntimeScope,
) {
    let (getter_expression, setter_statement) =
        resolve_component_bind_getter_setter(ast, &bind_expression, scope);
    properties.push(object_get_property(ast, bind_name, getter_expression));
    properties.push(object_set_property(ast, bind_name, setter_statement));
}

fn resolve_component_bind_getter_setter<'a>(
    ast: AstBuilder<'a>,
    bind_expression: &Expression<'a>,
    scope: &RuntimeScope,
) -> (Expression<'a>, Statement<'a>) {
    if let Expression::SequenceExpression(sequence) = strip_typescript_wrappers(bind_expression) {
        if let (Some(getter), Some(setter), None) = (
            sequence.expressions.first(),
            sequence.expressions.get(1),
            sequence.expressions.get(2),
        ) {
            let getter_expression = ast.expression_call(
                SPAN,
                resolve_expression(ast, getter.clone_in(ast.allocator), scope),
                NONE,
                ast.vec(),
                false,
            );
            let setter_callee = resolve_expression(ast, setter.clone_in(ast.allocator), scope);
            let setter_statement = ast.statement_expression(
                SPAN,
                ast.expression_call(
                    SPAN,
                    setter_callee,
                    NONE,
                    ast.vec1(ast.expression_identifier(SPAN, ast.ident("$$value")).into()),
                    false,
                ),
            );
            return (getter_expression, setter_statement);
        }
    }

    let getter_expression = resolve_expression(ast, bind_expression.clone_in(ast.allocator), scope);
    let setter_statement = build_component_bind_setter_statement(ast, bind_expression, scope)
        .unwrap_or_else(|| {
            ast.statement_expression(SPAN, ast.expression_identifier(SPAN, ast.ident("$$value")))
        });
    (getter_expression, setter_statement)
}

fn build_component_bind_setter_statement<'a>(
    ast: AstBuilder<'a>,
    expression: &Expression<'a>,
    scope: &RuntimeScope,
) -> Option<Statement<'a>> {
    build_component_assignment_statement(
        ast,
        expression,
        scope,
        ast.expression_identifier(SPAN, ast.ident("$$value")),
    )
}

fn build_component_bind_this_statement<'a>(
    ast: AstBuilder<'a>,
    expression: &Expression<'a>,
    scope: &RuntimeScope,
) -> Option<Statement<'a>> {
    if let Expression::SequenceExpression(sequence) = strip_typescript_wrappers(expression) {
        if let (Some(_getter), Some(setter), None) = (
            sequence.expressions.first(),
            sequence.expressions.get(1),
            sequence.expressions.get(2),
        ) {
            let setter_callee = resolve_expression(ast, setter.clone_in(ast.allocator), scope);
            return Some(
                ast.statement_expression(
                    SPAN,
                    ast.expression_call(
                        SPAN,
                        setter_callee,
                        NONE,
                        ast.vec1(
                            ast.expression_identifier(SPAN, ast.ident("__lux_component"))
                                .into(),
                        ),
                        false,
                    ),
                ),
            );
        }
    }

    build_component_assignment_statement(
        ast,
        expression,
        scope,
        ast.expression_identifier(SPAN, ast.ident("__lux_component")),
    )
}

fn build_component_assignment_statement<'a>(
    ast: AstBuilder<'a>,
    expression: &Expression<'a>,
    scope: &RuntimeScope,
    rhs: Expression<'a>,
) -> Option<Statement<'a>> {
    match strip_typescript_wrappers(expression) {
        Expression::Identifier(identifier) => {
            let assignment_target = if scope.contains(identifier.name.as_str()) {
                ast.simple_assignment_target_assignment_target_identifier(
                    SPAN,
                    ast.ident(identifier.name.as_str()),
                )
                .into()
            } else {
                ast.member_expression_static(
                    SPAN,
                    ast.expression_identifier(SPAN, ast.ident("_props")),
                    ast.identifier_name(SPAN, ast.ident(identifier.name.as_str())),
                    false,
                )
                .into()
            };
            Some(ast.statement_expression(
                SPAN,
                ast.expression_assignment(SPAN, AssignmentOperator::Assign, assignment_target, rhs),
            ))
        }
        Expression::StaticMemberExpression(member) => {
            let Expression::Identifier(root) = strip_typescript_wrappers(&member.object) else {
                return None;
            };
            if scope.contains(root.name.as_str()) {
                return Some(
                    ast.statement_expression(
                        SPAN,
                        ast.expression_assignment(
                            SPAN,
                            AssignmentOperator::Assign,
                            ast.member_expression_static(
                                SPAN,
                                ast.expression_identifier(SPAN, ast.ident(root.name.as_str())),
                                ast.identifier_name(SPAN, ast.ident(member.property.name.as_str())),
                                false,
                            )
                            .into(),
                            rhs,
                        ),
                    ),
                );
            }
            build_component_member_assign_statement(
                ast,
                root.name.as_str(),
                ast.member_expression_static(
                    SPAN,
                    ast.expression_identifier(SPAN, ast.ident("__lux_bind_target")),
                    ast.identifier_name(SPAN, ast.ident(member.property.name.as_str())),
                    false,
                )
                .into(),
                rhs,
            )
        }
        Expression::ComputedMemberExpression(member) => {
            let Expression::Identifier(root) = strip_typescript_wrappers(&member.object) else {
                return None;
            };
            let property = strip_typescript_wrappers(&member.expression).clone_in(ast.allocator);
            if scope.contains(root.name.as_str()) {
                return Some(
                    ast.statement_expression(
                        SPAN,
                        ast.expression_assignment(
                            SPAN,
                            AssignmentOperator::Assign,
                            ast.member_expression_computed(
                                SPAN,
                                ast.expression_identifier(SPAN, ast.ident(root.name.as_str())),
                                property,
                                false,
                            )
                            .into(),
                            rhs,
                        ),
                    ),
                );
            }
            build_component_member_assign_statement(
                ast,
                root.name.as_str(),
                ast.member_expression_computed(
                    SPAN,
                    ast.expression_identifier(SPAN, ast.ident("__lux_bind_target")),
                    property,
                    false,
                )
                .into(),
                rhs,
            )
        }
        _ => None,
    }
}

fn build_component_member_assign_statement<'a>(
    ast: AstBuilder<'a>,
    root_name: &str,
    assignment_target: oxc_ast::ast::AssignmentTarget<'a>,
    rhs: Expression<'a>,
) -> Option<Statement<'a>> {
    let target_decl = ast.variable_declarator(
        SPAN,
        VariableDeclarationKind::Const,
        ast.binding_pattern_binding_identifier(SPAN, ast.ident("__lux_bind_target")),
        NONE,
        Some(
            ast.member_expression_static(
                SPAN,
                ast.expression_identifier(SPAN, ast.ident("_props")),
                ast.identifier_name(SPAN, ast.ident(root_name)),
                false,
            )
            .into(),
        ),
        false,
    );
    let test = ast.expression_binary(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident("__lux_bind_target")),
        BinaryOperator::Inequality,
        ast.expression_null_literal(SPAN),
    );
    let assignment =
        ast.expression_assignment(SPAN, AssignmentOperator::Assign, assignment_target, rhs);
    let consequent =
        ast.statement_block(SPAN, ast.vec1(ast.statement_expression(SPAN, assignment)));
    Some(
        ast.statement_block(
            SPAN,
            ast.vec_from_array([
                ast.declaration_variable(
                    SPAN,
                    VariableDeclarationKind::Const,
                    ast.vec1(target_decl),
                    false,
                )
                .into(),
                ast.statement_if(SPAN, test, consequent, None),
            ]),
        ),
    )
}

fn object_get_property<'a>(
    ast: AstBuilder<'a>,
    name: &str,
    value_expression: Expression<'a>,
) -> oxc_ast::ast::ObjectPropertyKind<'a> {
    let (key, computed) = if super::scope::is_valid_js_identifier(name) {
        (
            ast.property_key_static_identifier(SPAN, ast.ident(name)),
            false,
        )
    } else {
        (string_expr(ast, name).into(), false)
    };
    let params =
        ast.alloc_formal_parameters(SPAN, FormalParameterKind::FormalParameter, ast.vec(), NONE);
    let body = ast.alloc_function_body(
        SPAN,
        ast.vec(),
        ast.vec1(ast.statement_return(SPAN, Some(value_expression))),
    );
    let function_expression = ast.expression_function(
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
    ast.object_property_kind_object_property(
        SPAN,
        PropertyKind::Get,
        key,
        function_expression,
        false,
        false,
        computed,
    )
}

fn object_set_property<'a>(
    ast: AstBuilder<'a>,
    name: &str,
    setter_statement: Statement<'a>,
) -> oxc_ast::ast::ObjectPropertyKind<'a> {
    let (key, computed) = if super::scope::is_valid_js_identifier(name) {
        (
            ast.property_key_static_identifier(SPAN, ast.ident(name)),
            false,
        )
    } else {
        (string_expr(ast, name).into(), false)
    };
    let params = ast.alloc_formal_parameters(
        SPAN,
        FormalParameterKind::FormalParameter,
        ast.vec1(ast.formal_parameter(
            SPAN,
            ast.vec(),
            ast.binding_pattern_binding_identifier(SPAN, ast.ident("$$value")),
            NONE,
            NONE,
            false,
            None,
            false,
            false,
        )),
        NONE,
    );
    let body = ast.alloc_function_body(SPAN, ast.vec(), ast.vec1(setter_statement));
    let function_expression = ast.expression_function(
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
    ast.object_property_kind_object_property(
        SPAN,
        PropertyKind::Set,
        key,
        function_expression,
        false,
        false,
        computed,
    )
}

fn strip_typescript_wrappers<'a>(expression: &'a Expression<'a>) -> &'a Expression<'a> {
    match expression {
        Expression::TSAsExpression(wrapper) => strip_typescript_wrappers(&wrapper.expression),
        Expression::TSSatisfiesExpression(wrapper) => {
            strip_typescript_wrappers(&wrapper.expression)
        }
        Expression::TSTypeAssertion(wrapper) => strip_typescript_wrappers(&wrapper.expression),
        Expression::TSNonNullExpression(wrapper) => strip_typescript_wrappers(&wrapper.expression),
        Expression::TSInstantiationExpression(wrapper) => {
            strip_typescript_wrappers(&wrapper.expression)
        }
        Expression::ParenthesizedExpression(wrapper) => {
            strip_typescript_wrappers(&wrapper.expression)
        }
        _ => expression,
    }
}

fn component_has_children_prop(attributes: &[AttributeNode<'_>]) -> bool {
    attributes.iter().any(|attribute| match attribute {
        AttributeNode::Attribute(attribute) => attribute.name == "children",
        AttributeNode::BindDirective(attribute) => attribute.name == "children",
        _ => false,
    })
}

fn build_component_event_forward_expression<'a>(
    ast: AstBuilder<'a>,
    event_name: &str,
) -> Expression<'a> {
    let events_object = ast.member_expression_static(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident("_props")),
        ast.identifier_name(SPAN, ast.ident("$$events")),
        false,
    );
    let event_lookup: Expression<'a> = if super::scope::is_valid_js_identifier(event_name) {
        ast.member_expression_static(
            SPAN,
            events_object.clone_in(ast.allocator).into(),
            ast.identifier_name(SPAN, ast.ident(event_name)),
            false,
        )
        .into()
    } else {
        ast.member_expression_computed(
            SPAN,
            events_object.clone_in(ast.allocator).into(),
            string_expr(ast, event_name),
            false,
        )
        .into()
    };
    ast.expression_logical(
        SPAN,
        events_object.into(),
        LogicalOperator::And,
        event_lookup,
    )
}

fn collect_default_slot_let_bindings<'a>(
    attributes: &'a [AttributeNode<'a>],
) -> Vec<SlotLetBinding<'a>> {
    let mut bindings = Vec::new();

    for attribute in attributes {
        if let AttributeNode::LetDirective(directive) = attribute {
            if let Some(binding) = slot_let_binding(directive) {
                if !bindings
                    .iter()
                    .any(|existing: &SlotLetBinding<'a>| existing.local_name == binding.local_name)
                {
                    bindings.push(binding);
                }
            }
        }
    }

    bindings
}

fn build_slot_function_for_nodes<'a>(
    ast: AstBuilder<'a>,
    nodes: &[&FragmentNode<'_>],
    let_bindings: &[SlotLetBinding<'_>],
    scope: &RuntimeScope,
) -> Expression<'a> {
    let mut slot_scope = scope.clone();
    let mut body_statements = ast.vec();

    if !let_bindings.is_empty() {
        let slot_props_ident = ast.expression_identifier(SPAN, ast.ident("__lux_slot_props"));

        for binding in let_bindings {
            let slot_prop_value = ast.expression_logical(
                SPAN,
                slot_props_ident.clone_in(ast.allocator),
                LogicalOperator::And,
                ast.member_expression_static(
                    SPAN,
                    slot_props_ident.clone_in(ast.allocator),
                    ast.identifier_name(SPAN, ast.ident(binding.prop_name)),
                    false,
                )
                .into(),
            );
            let declarator = ast.variable_declarator(
                SPAN,
                VariableDeclarationKind::Const,
                ast.binding_pattern_binding_identifier(SPAN, ast.ident(binding.local_name)),
                NONE,
                Some(slot_prop_value),
                false,
            );
            body_statements.push(
                ast.declaration_variable(
                    SPAN,
                    VariableDeclarationKind::Const,
                    ast.vec1(declarator),
                    false,
                )
                .into(),
            );
            slot_scope = slot_scope.with_name(binding.local_name);
        }
    }

    let child_expression = render_fragment_nodes_expression(ast, nodes, &slot_scope);
    body_statements.push(ast.statement_return(SPAN, Some(child_expression)));

    let params = if let_bindings.is_empty() {
        ast.alloc_formal_parameters(SPAN, FormalParameterKind::FormalParameter, ast.vec(), NONE)
    } else {
        ast.alloc_formal_parameters(
            SPAN,
            FormalParameterKind::FormalParameter,
            ast.vec1(ast.formal_parameter(
                SPAN,
                ast.vec(),
                ast.binding_pattern_binding_identifier(SPAN, ast.ident("__lux_slot_props")),
                NONE,
                NONE,
                false,
                None,
                false,
                false,
            )),
            NONE,
        )
    };
    let body = ast.alloc_function_body(SPAN, ast.vec(), body_statements);
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

#[derive(Clone, Copy)]
struct SlotLetBinding<'a> {
    prop_name: &'a str,
    local_name: &'a str,
}

struct ComponentSlotGroup<'a> {
    slot_name: &'a str,
    nodes: Vec<&'a FragmentNode<'a>>,
    let_bindings: Vec<SlotLetBinding<'a>>,
}

fn collect_component_slot_groups<'a>(fragment: &'a Fragment<'a>) -> Vec<ComponentSlotGroup<'a>> {
    let mut groups: Vec<ComponentSlotGroup<'a>> = Vec::new();

    for node in &fragment.nodes {
        let slot_name = component_child_slot_name(node).unwrap_or("default");
        let mut node_bindings = component_child_slot_let_bindings(node, slot_name);

        if let Some(group) = groups.iter_mut().find(|group| group.slot_name == slot_name) {
            group.nodes.push(node);
            for binding in node_bindings.drain(..) {
                if !group
                    .let_bindings
                    .iter()
                    .any(|existing| existing.local_name == binding.local_name)
                {
                    group.let_bindings.push(binding);
                }
            }
        } else {
            groups.push(ComponentSlotGroup {
                slot_name,
                nodes: vec![node],
                let_bindings: node_bindings,
            });
        }
    }

    groups
}

fn component_child_slot_let_bindings<'a>(
    node: &'a FragmentNode<'a>,
    slot_name: &str,
) -> Vec<SlotLetBinding<'a>> {
    let Some(attributes) = fragment_node_attributes(node) else {
        return Vec::new();
    };
    let has_slot_attribute = slot_attribute_name(attributes).is_some();
    let collect_from_node = has_slot_attribute || matches!(node, FragmentNode::SvelteFragment(_));
    if !collect_from_node {
        return Vec::new();
    }
    if slot_name == "default"
        && !has_slot_attribute
        && !matches!(node, FragmentNode::SvelteFragment(_))
    {
        return Vec::new();
    }

    let mut bindings = Vec::new();
    for attribute in attributes {
        if let AttributeNode::LetDirective(directive) = attribute {
            if let Some(binding) = slot_let_binding(directive) {
                if !bindings
                    .iter()
                    .any(|existing: &SlotLetBinding<'a>| existing.local_name == binding.local_name)
                {
                    bindings.push(binding);
                }
            }
        }
    }
    bindings
}

fn component_child_slot_name<'a>(node: &'a FragmentNode<'a>) -> Option<&'a str> {
    fragment_node_attributes(node).and_then(slot_attribute_name)
}

fn fragment_node_attributes<'a>(node: &'a FragmentNode<'a>) -> Option<&'a [AttributeNode<'a>]> {
    match node {
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
    }
}

fn slot_attribute_name<'a>(attributes: &'a [AttributeNode<'a>]) -> Option<&'a str> {
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

fn slot_let_binding<'a>(directive: &'a LetDirective<'a>) -> Option<SlotLetBinding<'a>> {
    match &directive.expression {
        None => Some(SlotLetBinding {
            prop_name: directive.name,
            local_name: directive.name,
        }),
        Some(Expression::Identifier(identifier)) => Some(SlotLetBinding {
            prop_name: directive.name,
            local_name: identifier.name.as_str(),
        }),
        Some(_) => None,
    }
}
