use lux_ast::template::attribute::{Attribute, AttributeNode, AttributeValue};
use lux_ast::template::directive::{
    AnimateDirective, ClassDirective, EventModifier, OnDirective, StyleDirective,
    StyleDirectiveValue, StyleModifier, TransitionDirective, UseDirective,
};
use lux_ast::template::tag::TextOrExpressionTag;
use lux_utils::attributes::is_boolean_attribute;
use oxc_allocator::CloneIn;
use oxc_ast::{
    AstBuilder, NONE,
    ast::{
        AssignmentOperator, BinaryOperator, Expression, FormalParameterKind, FunctionType,
        LogicalOperator, Statement, VariableDeclarationKind,
    },
};
use oxc_span::SPAN;

use super::expr::{
    escape_attr_expression, join_chunks_expression, string_expr, stringify_expression,
};
use super::scope::{RuntimeScope, is_valid_js_identifier, resolve_expression};

pub(super) fn render_attribute_expression<'a>(
    ast: AstBuilder<'a>,
    attribute: &AttributeNode<'_>,
    scope: &RuntimeScope,
    element_name: Option<&str>,
) -> Expression<'a> {
    match attribute {
        AttributeNode::Attribute(attribute) => {
            if is_event_attribute_name(attribute.name) {
                return string_expr(ast, "");
            }
            match &attribute.value {
                AttributeValue::True => string_expr(ast, &format!(" {}", attribute.name)),
                AttributeValue::ExpressionTag(tag) => render_named_expression_attribute(
                    ast,
                    attribute.name,
                    resolve_expression(ast, tag.expression.clone_in(ast.allocator), scope),
                ),
                AttributeValue::Sequence(chunks) => {
                    let mut value_parts = ast.vec();
                    for chunk in chunks {
                        let chunk_expr = match chunk {
                            TextOrExpressionTag::Text(text) => string_expr(ast, text.raw),
                            TextOrExpressionTag::ExpressionTag(tag) => stringify_expression(
                                ast,
                                resolve_expression(
                                    ast,
                                    tag.expression.clone_in(ast.allocator),
                                    scope,
                                ),
                            ),
                        };
                        value_parts.push(chunk_expr);
                    }

                    render_named_expression_attribute(
                        ast,
                        attribute.name,
                        join_chunks_expression(ast, value_parts),
                    )
                }
            }
        }
        AttributeNode::SpreadAttribute(attribute) => render_spread_attribute_expression(
            ast,
            resolve_expression(ast, attribute.expression.clone_in(ast.allocator), scope),
        ),
        AttributeNode::BindDirective(attribute) => render_bind_directive_attribute_expression(
            ast,
            attribute.name.to_string(),
            attribute.expression.clone_in(ast.allocator),
            scope,
            element_name,
        ),
        AttributeNode::ClassDirective(attribute) => render_class_directive_attribute_expression(
            ast,
            attribute.name,
            resolve_expression(ast, attribute.expression.clone_in(ast.allocator), scope),
        ),
        AttributeNode::StyleDirective(attribute) => {
            render_style_directive_attribute_expression(ast, attribute, scope)
        }
        AttributeNode::OnDirective(directive) => {
            render_on_directive_attribute_expression(ast, directive, scope)
        }
        AttributeNode::UseDirective(directive) => {
            render_use_directive_attribute_expression(ast, directive, scope)
        }
        AttributeNode::TransitionDirective(directive) => {
            render_transition_directive_attribute_expression(ast, directive, scope)
        }
        AttributeNode::AnimateDirective(directive) => {
            render_animate_directive_attribute_expression(ast, directive, scope)
        }
        AttributeNode::AttachTag(tag) => render_attach_tag_attribute_expression(ast, tag, scope),
        AttributeNode::LetDirective(_) => string_expr(ast, ""),
    }
}

pub(super) fn render_element_attribute_chunks<'a>(
    ast: AstBuilder<'a>,
    attributes: &[AttributeNode<'_>],
    scope: &RuntimeScope,
    element_name: Option<&str>,
) -> oxc_allocator::Vec<'a, Expression<'a>> {
    let mut class_attributes: Vec<&Attribute<'_>> = Vec::new();
    let mut class_directives: Vec<&ClassDirective<'_>> = Vec::new();
    let mut style_attributes: Vec<&Attribute<'_>> = Vec::new();
    let mut style_directives: Vec<&StyleDirective<'_>> = Vec::new();
    let mut spread_count = 0usize;

    for attribute in attributes {
        match attribute {
            AttributeNode::Attribute(attribute) if attribute.name == "class" => {
                class_attributes.push(attribute);
            }
            AttributeNode::Attribute(attribute) if attribute.name == "style" => {
                style_attributes.push(attribute);
            }
            AttributeNode::ClassDirective(directive) => class_directives.push(directive),
            AttributeNode::StyleDirective(directive) => style_directives.push(directive),
            AttributeNode::SpreadAttribute(_) => spread_count += 1,
            _ => {}
        }
    }

    let has_spread = spread_count > 0;
    let can_merge_into_spread = spread_count == 1
        && class_attributes.len() <= 1
        && style_attributes.len() <= 1
        && (!class_attributes.is_empty()
            || !class_directives.is_empty()
            || !style_attributes.is_empty()
            || !style_directives.is_empty());
    let can_merge_class = !can_merge_into_spread
        && !has_spread
        && !class_directives.is_empty()
        && class_attributes.len() <= 1;
    let can_merge_style = !can_merge_into_spread
        && !has_spread
        && !style_directives.is_empty()
        && style_attributes.len() <= 1;

    let mut chunks = ast.vec_with_capacity(attributes.len() + 2);
    let mut emitted_merged_spread = false;
    for attribute in attributes {
        if can_merge_into_spread {
            match attribute {
                AttributeNode::SpreadAttribute(spread) => {
                    if !emitted_merged_spread {
                        chunks.push(render_merged_spread_attribute_expression(
                            ast,
                            resolve_expression(
                                ast,
                                spread.expression.clone_in(ast.allocator),
                                scope,
                            ),
                            class_attributes.first().copied(),
                            &class_directives,
                            style_attributes.first().copied(),
                            &style_directives,
                            scope,
                        ));
                        emitted_merged_spread = true;
                    }
                    continue;
                }
                AttributeNode::Attribute(attribute)
                    if attribute.name == "class" || attribute.name == "style" =>
                {
                    continue;
                }
                AttributeNode::ClassDirective(_) | AttributeNode::StyleDirective(_) => continue,
                _ => {}
            }
        }

        if can_merge_class {
            match attribute {
                AttributeNode::Attribute(attribute) if attribute.name == "class" => continue,
                AttributeNode::ClassDirective(_) => continue,
                _ => {}
            }
        }
        if can_merge_style {
            match attribute {
                AttributeNode::Attribute(attribute) if attribute.name == "style" => continue,
                AttributeNode::StyleDirective(_) => continue,
                _ => {}
            }
        }
        chunks.push(render_attribute_expression(
            ast,
            attribute,
            scope,
            element_name,
        ));
    }

    if can_merge_class {
        chunks.push(render_merged_class_attribute_expression(
            ast,
            class_attributes.first().copied(),
            &class_directives,
            scope,
        ));
    }
    if can_merge_style {
        chunks.push(render_merged_style_attribute_expression(
            ast,
            style_attributes.first().copied(),
            &style_directives,
            scope,
        ));
    }

    chunks
}

pub(super) fn render_target_attribute_expression<'a>(
    ast: AstBuilder<'a>,
    attribute: &AttributeNode<'_>,
    scope: &RuntimeScope,
    target_name: &str,
) -> Expression<'a> {
    match attribute {
        AttributeNode::OnDirective(directive) => {
            render_on_target_directive_attribute_expression(ast, directive, scope, target_name)
        }
        AttributeNode::BindDirective(directive) => {
            render_bind_target_directive_attribute_expression(
                ast,
                directive.name,
                directive.expression.clone_in(ast.allocator),
                scope,
                target_name,
            )
        }
        _ => string_expr(ast, ""),
    }
}

fn render_named_expression_attribute<'a>(
    ast: AstBuilder<'a>,
    name: &str,
    value: Expression<'a>,
) -> Expression<'a> {
    ast.expression_call(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident("__lux_attr")),
        NONE,
        ast.vec_from_array([
            string_expr(ast, name).into(),
            value.into(),
            ast.expression_boolean_literal(SPAN, is_boolean_attribute(name))
                .into(),
        ]),
        false,
    )
}

fn render_merged_class_attribute_expression<'a>(
    ast: AstBuilder<'a>,
    class_attribute: Option<&Attribute<'_>>,
    class_directives: &[&ClassDirective<'_>],
    scope: &RuntimeScope,
) -> Expression<'a> {
    let base_class = class_attribute.map_or_else(
        || ast.expression_null_literal(SPAN),
        |attribute| attribute_value_expression(ast, &attribute.value, scope),
    );

    let mut toggles = ast.vec_with_capacity(class_directives.len());
    for directive in class_directives {
        let value = resolve_expression(ast, directive.expression.clone_in(ast.allocator), scope);
        toggles.push(object_init_property(ast, directive.name, value));
    }

    let merged = ast.expression_call(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident("__lux_class_attr")),
        NONE,
        ast.vec_from_array([
            base_class.into(),
            ast.expression_object(SPAN, toggles).into(),
        ]),
        false,
    );
    render_named_expression_attribute(ast, "class", merged)
}

fn render_merged_style_attribute_expression<'a>(
    ast: AstBuilder<'a>,
    style_attribute: Option<&Attribute<'_>>,
    style_directives: &[&StyleDirective<'_>],
    scope: &RuntimeScope,
) -> Expression<'a> {
    let base_style = style_attribute.map_or_else(
        || ast.expression_null_literal(SPAN),
        |attribute| attribute_value_expression(ast, &attribute.value, scope),
    );

    let mut styles = ast.vec_with_capacity(style_directives.len());
    for directive in style_directives {
        let value = render_style_directive_value_expression(ast, directive, scope);
        let style_body = if directive.modifiers.contains(&StyleModifier::Important) {
            join_chunks_expression(
                ast,
                ast.vec_from_array([
                    value.clone_in(ast.allocator),
                    string_expr(ast, " !important"),
                ]),
            )
        } else {
            value.clone_in(ast.allocator)
        };
        let omit = is_falsy_attribute_value_expression(ast, value);
        let style_value =
            ast.expression_conditional(SPAN, omit, ast.expression_null_literal(SPAN), style_body);
        styles.push(object_init_property(ast, directive.name, style_value));
    }

    let merged = ast.expression_call(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident("__lux_style_attr")),
        NONE,
        ast.vec_from_array([
            base_style.into(),
            ast.expression_object(SPAN, styles).into(),
        ]),
        false,
    );
    render_named_expression_attribute(ast, "style", merged)
}

fn attribute_value_expression<'a>(
    ast: AstBuilder<'a>,
    value: &AttributeValue<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    match value {
        AttributeValue::True => string_expr(ast, ""),
        AttributeValue::ExpressionTag(tag) => {
            resolve_expression(ast, tag.expression.clone_in(ast.allocator), scope)
        }
        AttributeValue::Sequence(chunks) => {
            let mut value_parts = ast.vec_with_capacity(chunks.len());
            for chunk in chunks {
                let value = match chunk {
                    TextOrExpressionTag::Text(text) => string_expr(ast, text.raw),
                    TextOrExpressionTag::ExpressionTag(tag) => stringify_expression(
                        ast,
                        resolve_expression(ast, tag.expression.clone_in(ast.allocator), scope),
                    ),
                };
                value_parts.push(value);
            }
            join_chunks_expression(ast, value_parts)
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
        oxc_ast::ast::PropertyKind::Init,
        key,
        value,
        false,
        false,
        computed,
    )
}

fn render_class_directive_attribute_expression<'a>(
    ast: AstBuilder<'a>,
    name: &str,
    value: Expression<'a>,
) -> Expression<'a> {
    let class_attr = join_chunks_expression(
        ast,
        ast.vec_from_array([
            string_expr(ast, " class=\""),
            string_expr(ast, name),
            string_expr(ast, "\""),
        ]),
    );
    ast.expression_conditional(SPAN, value, class_attr, string_expr(ast, ""))
}

fn render_style_directive_attribute_expression<'a>(
    ast: AstBuilder<'a>,
    directive: &StyleDirective<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let value = render_style_directive_value_expression(ast, directive, scope);
    let style_body = if directive.modifiers.contains(&StyleModifier::Important) {
        join_chunks_expression(
            ast,
            ast.vec_from_array([
                value.clone_in(ast.allocator),
                string_expr(ast, " !important"),
            ]),
        )
    } else {
        value.clone_in(ast.allocator)
    };
    let style_attr = join_chunks_expression(
        ast,
        ast.vec_from_array([
            string_expr(ast, &format!(" style=\"{}: ", directive.name)),
            escape_attr_expression(ast, stringify_expression(ast, style_body)),
            string_expr(ast, ";\""),
        ]),
    );

    let omit = is_falsy_attribute_value_expression(ast, value);
    ast.expression_conditional(SPAN, omit, string_expr(ast, ""), style_attr)
}

fn render_style_directive_value_expression<'a>(
    ast: AstBuilder<'a>,
    directive: &StyleDirective<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    match &directive.value {
        StyleDirectiveValue::True => {
            if is_valid_js_identifier(directive.name) {
                resolve_expression(
                    ast,
                    ast.expression_identifier(SPAN, ast.ident(directive.name)),
                    scope,
                )
            } else {
                string_expr(ast, "")
            }
        }
        StyleDirectiveValue::ExpressionTag(tag) => {
            resolve_expression(ast, tag.expression.clone_in(ast.allocator), scope)
        }
        StyleDirectiveValue::Sequence(chunks) => {
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

fn render_spread_attribute_expression<'a>(
    ast: AstBuilder<'a>,
    spread_expression: Expression<'a>,
) -> Expression<'a> {
    ast.expression_call(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident("__lux_attributes")),
        NONE,
        ast.vec1(spread_expression.into()),
        false,
    )
}

fn render_merged_spread_attribute_expression<'a>(
    ast: AstBuilder<'a>,
    spread_expression: Expression<'a>,
    class_attribute: Option<&Attribute<'_>>,
    class_directives: &[&ClassDirective<'_>],
    style_attribute: Option<&Attribute<'_>>,
    style_directives: &[&StyleDirective<'_>],
    scope: &RuntimeScope,
) -> Expression<'a> {
    let class_toggles = if class_directives.is_empty() {
        ast.expression_null_literal(SPAN)
    } else {
        build_class_directive_object_expression(ast, class_directives, scope)
    };
    let style_values = if style_directives.is_empty() {
        ast.expression_null_literal(SPAN)
    } else {
        build_style_directive_object_expression(ast, style_directives, scope)
    };
    let class_base = class_attribute.map_or_else(
        || ast.expression_null_literal(SPAN),
        |attribute| attribute_value_expression(ast, &attribute.value, scope),
    );
    let style_base = style_attribute.map_or_else(
        || ast.expression_null_literal(SPAN),
        |attribute| attribute_value_expression(ast, &attribute.value, scope),
    );

    ast.expression_call(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident("__lux_attributes")),
        NONE,
        ast.vec_from_array([
            spread_expression.into(),
            class_toggles.into(),
            style_values.into(),
            class_base.into(),
            style_base.into(),
        ]),
        false,
    )
}

fn build_class_directive_object_expression<'a>(
    ast: AstBuilder<'a>,
    class_directives: &[&ClassDirective<'_>],
    scope: &RuntimeScope,
) -> Expression<'a> {
    let mut toggles = ast.vec_with_capacity(class_directives.len());
    for directive in class_directives {
        let value = resolve_expression(ast, directive.expression.clone_in(ast.allocator), scope);
        toggles.push(object_init_property(ast, directive.name, value));
    }
    ast.expression_object(SPAN, toggles)
}

fn build_style_directive_object_expression<'a>(
    ast: AstBuilder<'a>,
    style_directives: &[&StyleDirective<'_>],
    scope: &RuntimeScope,
) -> Expression<'a> {
    let mut styles = ast.vec_with_capacity(style_directives.len());
    for directive in style_directives {
        let value = render_style_directive_value_expression(ast, directive, scope);
        let style_body = if directive.modifiers.contains(&StyleModifier::Important) {
            join_chunks_expression(
                ast,
                ast.vec_from_array([
                    value.clone_in(ast.allocator),
                    string_expr(ast, " !important"),
                ]),
            )
        } else {
            value.clone_in(ast.allocator)
        };
        let omit = is_falsy_attribute_value_expression(ast, value);
        let style_value =
            ast.expression_conditional(SPAN, omit, ast.expression_null_literal(SPAN), style_body);
        styles.push(object_init_property(ast, directive.name, style_value));
    }
    ast.expression_object(SPAN, styles)
}

fn render_bind_directive_attribute_expression<'a>(
    ast: AstBuilder<'a>,
    bind_name: String,
    bind_expression: Expression<'a>,
    scope: &RuntimeScope,
    element_name: Option<&str>,
) -> Expression<'a> {
    let (getter_expression, setter_expression) =
        resolve_bind_getter_setter_expression(ast, &bind_expression, scope);
    let bind_marker = ast.expression_call(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident("__lux_bind_attr")),
        NONE,
        ast.vec_from_array([
            string_expr(ast, bind_name.as_str()).into(),
            getter_expression.clone_in(ast.allocator).into(),
            setter_expression.into(),
        ]),
        false,
    );

    if !bind_name_emits_html_attribute(bind_name.as_str(), element_name) {
        return bind_marker;
    }

    let rendered_attr =
        render_named_expression_attribute(ast, bind_name.as_str(), getter_expression);
    join_chunks_expression(ast, ast.vec_from_array([rendered_attr, bind_marker]))
}

fn resolve_bind_getter_setter_expression<'a>(
    ast: AstBuilder<'a>,
    bind_expression: &Expression<'a>,
    scope: &RuntimeScope,
) -> (Expression<'a>, Expression<'a>) {
    if let Expression::SequenceExpression(sequence) = strip_typescript_wrappers(bind_expression) {
        if let (Some(getter), Some(setter), None) = (
            sequence.expressions.first(),
            sequence.expressions.get(1),
            sequence.expressions.get(2),
        ) {
            return (
                resolve_expression(ast, getter.clone_in(ast.allocator), scope),
                resolve_expression(ast, setter.clone_in(ast.allocator), scope),
            );
        }
    }

    (
        resolve_expression(ast, bind_expression.clone_in(ast.allocator), scope),
        build_bind_setter_expression(ast, bind_expression, scope),
    )
}

fn bind_name_emits_html_attribute(bind_name: &str, element_name: Option<&str>) -> bool {
    match bind_name {
        "value" => !matches!(element_name, Some("select")),
        "checked" => true,
        "open" => true,
        _ => false,
    }
}

fn build_bind_setter_expression<'a>(
    ast: AstBuilder<'a>,
    expression: &Expression<'a>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    if let Some(statement) = build_bind_setter_statement(ast, expression, scope) {
        let mut statements = ast.vec();
        statements.push(statement);
        statements.push(ast.statement_return(
            SPAN,
            Some(ast.expression_identifier(SPAN, ast.ident("__lux_value"))),
        ));
        return build_bind_setter_function(ast, statements);
    }

    build_bind_setter_function(
        ast,
        ast.vec1(ast.statement_return(
            SPAN,
            Some(ast.expression_identifier(SPAN, ast.ident("__lux_value"))),
        )),
    )
}

fn build_bind_setter_statement<'a>(
    ast: AstBuilder<'a>,
    expression: &Expression<'a>,
    scope: &RuntimeScope,
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
            let assignment = ast.expression_assignment(
                SPAN,
                AssignmentOperator::Assign,
                assignment_target,
                ast.expression_identifier(SPAN, ast.ident("__lux_value")),
            );
            Some(ast.statement_expression(SPAN, assignment))
        }
        Expression::StaticMemberExpression(member) => {
            let Expression::Identifier(root) = strip_typescript_wrappers(&member.object) else {
                return None;
            };
            if scope.contains(root.name.as_str()) {
                let assign = ast.expression_assignment(
                    SPAN,
                    AssignmentOperator::Assign,
                    ast.member_expression_static(
                        SPAN,
                        ast.expression_identifier(SPAN, ast.ident(root.name.as_str())),
                        ast.identifier_name(SPAN, ast.ident(member.property.name.as_str())),
                        false,
                    )
                    .into(),
                    ast.expression_identifier(SPAN, ast.ident("__lux_value")),
                );
                return Some(ast.statement_expression(SPAN, assign));
            }

            build_props_member_assign_statement(
                ast,
                root.name.as_str(),
                ast.member_expression_static(
                    SPAN,
                    ast.expression_identifier(SPAN, ast.ident("__lux_bind_target")),
                    ast.identifier_name(SPAN, ast.ident(member.property.name.as_str())),
                    false,
                )
                .into(),
            )
        }
        Expression::ComputedMemberExpression(member) => {
            let Expression::Identifier(root) = strip_typescript_wrappers(&member.object) else {
                return None;
            };
            let property = strip_typescript_wrappers(&member.expression).clone_in(ast.allocator);

            if scope.contains(root.name.as_str()) {
                let assign = ast.expression_assignment(
                    SPAN,
                    AssignmentOperator::Assign,
                    ast.member_expression_computed(
                        SPAN,
                        ast.expression_identifier(SPAN, ast.ident(root.name.as_str())),
                        property,
                        false,
                    )
                    .into(),
                    ast.expression_identifier(SPAN, ast.ident("__lux_value")),
                );
                return Some(ast.statement_expression(SPAN, assign));
            }

            build_props_member_assign_statement(
                ast,
                root.name.as_str(),
                ast.member_expression_computed(
                    SPAN,
                    ast.expression_identifier(SPAN, ast.ident("__lux_bind_target")),
                    property,
                    false,
                )
                .into(),
            )
        }
        _ => None,
    }
}

fn build_props_member_assign_statement<'a>(
    ast: AstBuilder<'a>,
    root_name: &str,
    assignment_target: oxc_ast::ast::AssignmentTarget<'a>,
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
    let assign = ast.expression_assignment(
        SPAN,
        AssignmentOperator::Assign,
        assignment_target,
        ast.expression_identifier(SPAN, ast.ident("__lux_value")),
    );
    let consequent = ast.statement_block(SPAN, ast.vec1(ast.statement_expression(SPAN, assign)));
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

fn build_bind_setter_function<'a>(
    ast: AstBuilder<'a>,
    statements: oxc_allocator::Vec<'a, Statement<'a>>,
) -> Expression<'a> {
    let params = ast.alloc_formal_parameters(
        SPAN,
        FormalParameterKind::FormalParameter,
        ast.vec1(ast.formal_parameter(
            SPAN,
            ast.vec(),
            ast.binding_pattern_binding_identifier(SPAN, ast.ident("__lux_value")),
            NONE,
            NONE,
            false,
            None,
            false,
            false,
        )),
        NONE,
    );
    let body = ast.alloc_function_body(SPAN, ast.vec(), statements);
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

fn render_on_directive_attribute_expression<'a>(
    ast: AstBuilder<'a>,
    directive: &OnDirective<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let Some(expression) = &directive.expression else {
        return string_expr(ast, "");
    };

    let handler = resolve_expression(ast, expression.clone_in(ast.allocator), scope);
    let mut modifiers = ast.vec_with_capacity(directive.modifiers.len());
    for modifier in &directive.modifiers {
        modifiers.push(string_expr(ast, event_modifier_name(*modifier)).into());
    }

    ast.expression_call(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident("__lux_event_attr")),
        NONE,
        ast.vec_from_array([
            string_expr(ast, directive.name).into(),
            handler.into(),
            ast.expression_array(SPAN, modifiers).into(),
        ]),
        false,
    )
}

fn render_on_target_directive_attribute_expression<'a>(
    ast: AstBuilder<'a>,
    directive: &OnDirective<'_>,
    scope: &RuntimeScope,
    target_name: &str,
) -> Expression<'a> {
    let Some(expression) = &directive.expression else {
        return string_expr(ast, "");
    };

    let handler = resolve_expression(ast, expression.clone_in(ast.allocator), scope);
    let mut modifiers = ast.vec_with_capacity(directive.modifiers.len());
    for modifier in &directive.modifiers {
        modifiers.push(string_expr(ast, event_modifier_name(*modifier)).into());
    }

    ast.expression_call(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident("__lux_event_target_attr")),
        NONE,
        ast.vec_from_array([
            string_expr(ast, target_name).into(),
            string_expr(ast, directive.name).into(),
            handler.into(),
            ast.expression_array(SPAN, modifiers).into(),
        ]),
        false,
    )
}

fn render_bind_target_directive_attribute_expression<'a>(
    ast: AstBuilder<'a>,
    bind_name: &str,
    bind_expression: Expression<'a>,
    scope: &RuntimeScope,
    target_name: &str,
) -> Expression<'a> {
    let (getter_expression, setter_expression) =
        resolve_bind_getter_setter_expression(ast, &bind_expression, scope);
    ast.expression_call(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident("__lux_bind_target_attr")),
        NONE,
        ast.vec_from_array([
            string_expr(ast, target_name).into(),
            string_expr(ast, bind_name).into(),
            getter_expression.into(),
            setter_expression.into(),
        ]),
        false,
    )
}

fn render_use_directive_attribute_expression<'a>(
    ast: AstBuilder<'a>,
    directive: &UseDirective<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let Some(name_expression) = directive_name_expression(ast, directive.name) else {
        return string_expr(ast, "");
    };

    let action_expression = resolve_expression(ast, name_expression, scope);
    let parameter_expression = directive.expression.as_ref().map_or_else(
        || ast.expression_identifier(SPAN, ast.ident("undefined")),
        |expression| resolve_expression(ast, expression.clone_in(ast.allocator), scope),
    );

    ast.expression_call(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident("__lux_use_attr")),
        NONE,
        ast.vec_from_array([
            string_expr(ast, directive.name).into(),
            action_expression.into(),
            parameter_expression.into(),
        ]),
        false,
    )
}

fn render_transition_directive_attribute_expression<'a>(
    ast: AstBuilder<'a>,
    directive: &TransitionDirective<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let Some(name_expression) = directive_name_expression(ast, directive.name) else {
        return string_expr(ast, "");
    };

    let transition_expression = resolve_expression(ast, name_expression, scope);
    let parameter_expression = directive.expression.as_ref().map_or_else(
        || ast.expression_identifier(SPAN, ast.ident("undefined")),
        |expression| resolve_expression(ast, expression.clone_in(ast.allocator), scope),
    );

    ast.expression_call(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident("__lux_transition_attr")),
        NONE,
        ast.vec_from_array([
            string_expr(ast, directive.name).into(),
            transition_expression.into(),
            parameter_expression.into(),
            ast.expression_boolean_literal(SPAN, directive.intro).into(),
            ast.expression_boolean_literal(SPAN, directive.outro).into(),
        ]),
        false,
    )
}

fn render_animate_directive_attribute_expression<'a>(
    ast: AstBuilder<'a>,
    directive: &AnimateDirective<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let Some(name_expression) = directive_name_expression(ast, directive.name) else {
        return string_expr(ast, "");
    };

    let animate_expression = resolve_expression(ast, name_expression, scope);
    let parameter_expression = directive.expression.as_ref().map_or_else(
        || ast.expression_identifier(SPAN, ast.ident("undefined")),
        |expression| resolve_expression(ast, expression.clone_in(ast.allocator), scope),
    );

    ast.expression_call(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident("__lux_animate_attr")),
        NONE,
        ast.vec_from_array([
            string_expr(ast, directive.name).into(),
            animate_expression.into(),
            parameter_expression.into(),
        ]),
        false,
    )
}

fn render_attach_tag_attribute_expression<'a>(
    ast: AstBuilder<'a>,
    tag: &lux_ast::template::tag::AttachTag<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let attach_expression = resolve_expression(ast, tag.expression.clone_in(ast.allocator), scope);
    ast.expression_call(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident("__lux_use_attr")),
        NONE,
        ast.vec_from_array([
            string_expr(ast, "attach").into(),
            attach_expression.into(),
            ast.expression_identifier(SPAN, ast.ident("undefined"))
                .into(),
        ]),
        false,
    )
}

fn directive_name_expression<'a>(ast: AstBuilder<'a>, name: &str) -> Option<Expression<'a>> {
    let mut segments = name.split('.');
    let first = segments.next()?;
    if !is_valid_js_identifier(first) {
        return None;
    }

    let mut expression = ast.expression_identifier(SPAN, ast.ident(first));
    for segment in segments {
        if !is_valid_js_identifier(segment) {
            return None;
        }
        expression = ast
            .member_expression_static(
                SPAN,
                expression,
                ast.identifier_name(SPAN, ast.ident(segment)),
                false,
            )
            .into();
    }
    Some(expression)
}

fn event_modifier_name(modifier: EventModifier) -> &'static str {
    match modifier {
        EventModifier::Capture => "capture",
        EventModifier::Nonpassive => "nonpassive",
        EventModifier::Once => "once",
        EventModifier::Passive => "passive",
        EventModifier::PreventDefault => "preventDefault",
        EventModifier::Self_ => "self",
        EventModifier::StopImmediatePropagation => "stopImmediatePropagation",
        EventModifier::StopPropagation => "stopPropagation",
        EventModifier::Trusted => "trusted",
    }
}

fn is_falsy_attribute_value_expression<'a>(
    ast: AstBuilder<'a>,
    value: Expression<'a>,
) -> Expression<'a> {
    let is_nullish = ast.expression_binary(
        SPAN,
        value.clone_in(ast.allocator),
        BinaryOperator::Equality,
        ast.expression_null_literal(SPAN),
    );
    let is_false = ast.expression_binary(
        SPAN,
        value,
        BinaryOperator::StrictEquality,
        ast.expression_boolean_literal(SPAN, false),
    );
    ast.expression_logical(SPAN, is_nullish, LogicalOperator::Or, is_false)
}

fn is_event_attribute_name(name: &str) -> bool {
    name.len() > 2 && name.starts_with("on")
}
