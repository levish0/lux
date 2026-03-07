use oxc_allocator::CloneIn;
use oxc_ast::{
    AstBuilder, NONE,
    ast::{
        AssignmentOperator, BinaryOperator, ExportDefaultDeclarationKind, Expression,
        FormalParameterKind, FunctionType, ImportOrExportKind, LogicalOperator, PropertyKind,
        Statement,
    },
};
use oxc_span::SPAN;

use super::consts::{
    LUX_BEGIN_RENDER, LUX_CLEANUP_MOUNT, LUX_CSS, LUX_CSS_HASH, LUX_CSS_SCOPE, LUX_END_RENDER,
    LUX_HAS_DYNAMIC, LUX_IS_MOUNT_TARGET, LUX_MOUNT_ACTIONS, LUX_MOUNT_ANIMATIONS,
    LUX_MOUNT_BINDINGS, LUX_MOUNT_EVENTS, LUX_MOUNT_HEAD, LUX_MOUNT_HTML, LUX_MOUNT_TRANSITIONS,
    LUX_TEMPLATE,
};

pub(super) fn named_export_statement(ast: AstBuilder) -> Statement {
    let mut specifiers = ast.vec_with_capacity(5);
    specifiers.push(export_specifier(ast, LUX_TEMPLATE, "template"));
    specifiers.push(export_specifier(ast, LUX_CSS, "css"));
    specifiers.push(export_specifier(ast, LUX_CSS_HASH, "cssHash"));
    specifiers.push(export_specifier(ast, LUX_CSS_SCOPE, "cssScope"));
    specifiers.push(export_specifier(ast, LUX_HAS_DYNAMIC, "hasDynamic"));

    ast.module_declaration_export_named_declaration(
        SPAN,
        None,
        specifiers,
        None,
        ImportOrExportKind::Value,
        NONE,
    )
    .into()
}

pub(super) fn default_export_statement<'a>(
    ast: AstBuilder<'a>,
    render_expression: Expression<'a>,
    render_setup_statements: oxc_allocator::Vec<'a, Statement<'a>>,
    head_expression: Option<Expression<'a>>,
    head_setup_statements: Option<oxc_allocator::Vec<'a, Statement<'a>>>,
) -> Statement<'a> {
    let mut properties = ast.vec_with_capacity(if head_expression.is_some() { 7 } else { 6 });
    properties.push(named_property(ast, "template", LUX_TEMPLATE));
    properties.push(named_property(ast, "css", LUX_CSS));
    properties.push(named_property(ast, "cssHash", LUX_CSS_HASH));
    properties.push(named_property(ast, "cssScope", LUX_CSS_SCOPE));
    properties.push(named_property(ast, "hasDynamic", LUX_HAS_DYNAMIC));
    properties.push(function_property(
        ast,
        "render",
        render_expression,
        render_setup_statements,
    ));
    if let (Some(head_expression), Some(head_setup_statements)) =
        (head_expression, head_setup_statements)
    {
        properties.push(function_property(
            ast,
            "head",
            head_expression,
            head_setup_statements,
        ));
    }

    let object_expression = ast.expression_object(SPAN, properties);
    ast.module_declaration_export_default_declaration(
        SPAN,
        ExportDefaultDeclarationKind::from(object_expression),
    )
    .into()
}

pub(super) fn client_default_export_statement<'a>(
    ast: AstBuilder<'a>,
    render_expression: Expression<'a>,
    render_setup_statements: oxc_allocator::Vec<'a, Statement<'a>>,
    head_expression: Option<Expression<'a>>,
) -> Statement<'a> {
    let params = ast.alloc_formal_parameters(
        SPAN,
        FormalParameterKind::FormalParameter,
        ast.vec_from_array([
            ast.formal_parameter(
                SPAN,
                ast.vec(),
                ast.binding_pattern_binding_identifier(SPAN, ast.ident("$$anchor")),
                NONE,
                NONE,
                false,
                None,
                false,
                false,
            ),
            ast.formal_parameter(
                SPAN,
                ast.vec(),
                ast.binding_pattern_binding_identifier(SPAN, ast.ident("$$props")),
                NONE,
                NONE,
                false,
                None,
                false,
                false,
            ),
        ]),
        NONE,
    );

    let anchor_ident = ast.expression_identifier(SPAN, ast.ident("$$anchor"));
    let props_ident = ast.expression_identifier(SPAN, ast.ident("$$props"));
    let mount_mode_call = ast.expression_call(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident(LUX_IS_MOUNT_TARGET)),
        NONE,
        ast.vec1(anchor_ident.clone_in(ast.allocator).into()),
        false,
    );
    let mount_mode_decl = ast.variable_declarator(
        SPAN,
        oxc_ast::ast::VariableDeclarationKind::Const,
        ast.binding_pattern_binding_identifier(SPAN, ast.ident("__lux_mount_mode")),
        NONE,
        Some(mount_mode_call),
        false,
    );

    let coalesced_props = ast.expression_logical(
        SPAN,
        props_ident,
        LogicalOperator::Coalesce,
        ast.expression_object(SPAN, ast.vec()),
    );
    let coalesced_anchor_props = ast.expression_logical(
        SPAN,
        anchor_ident.clone_in(ast.allocator),
        LogicalOperator::Coalesce,
        ast.expression_object(SPAN, ast.vec()),
    );
    let selected_props = ast.expression_conditional(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident("__lux_mount_mode")),
        coalesced_props,
        coalesced_anchor_props,
    );
    let props_decl = ast.variable_declarator(
        SPAN,
        oxc_ast::ast::VariableDeclarationKind::Const,
        ast.binding_pattern_binding_identifier(SPAN, ast.ident("_props")),
        NONE,
        Some(selected_props),
        false,
    );

    let self_member = ast.member_expression_static(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident("_props")),
        ast.identifier_name(SPAN, ast.ident("__lux_self")),
        false,
    );
    let self_missing = ast.expression_binary(
        SPAN,
        self_member.clone_in(ast.allocator).into(),
        BinaryOperator::Equality,
        ast.expression_null_literal(SPAN),
    );
    let self_assign = ast.expression_assignment(
        SPAN,
        AssignmentOperator::Assign,
        self_member.into(),
        ast.expression_identifier(SPAN, ast.ident("__lux_component")),
    );
    let init_self = ast.expression_logical(SPAN, self_missing, LogicalOperator::And, self_assign);

    let render_state_decl = ast.variable_declarator(
        SPAN,
        oxc_ast::ast::VariableDeclarationKind::Const,
        ast.binding_pattern_binding_identifier(SPAN, ast.ident("__lux_render_state")),
        NONE,
        Some(ast.expression_call(
            SPAN,
            ast.expression_identifier(SPAN, ast.ident(LUX_BEGIN_RENDER)),
            NONE,
            ast.vec(),
            false,
        )),
        false,
    );
    let html_decl = ast.variable_declarator(
        SPAN,
        oxc_ast::ast::VariableDeclarationKind::Const,
        ast.binding_pattern_binding_identifier(SPAN, ast.ident("__lux_html")),
        NONE,
        Some(render_expression),
        false,
    );
    let head_html_decl = head_expression.map(|expression| {
        ast.variable_declarator(
            SPAN,
            oxc_ast::ast::VariableDeclarationKind::Const,
            ast.binding_pattern_binding_identifier(SPAN, ast.ident("__lux_head_html")),
            NONE,
            Some(expression),
            false,
        )
    });
    let events_decl = ast.variable_declarator(
        SPAN,
        oxc_ast::ast::VariableDeclarationKind::Const,
        ast.binding_pattern_binding_identifier(SPAN, ast.ident("__lux_events")),
        NONE,
        Some(
            ast.member_expression_static(
                SPAN,
                ast.expression_identifier(SPAN, ast.ident("__lux_render_result")),
                ast.identifier_name(SPAN, ast.ident("events")),
                false,
            )
            .into(),
        ),
        false,
    );
    let bindings_decl = ast.variable_declarator(
        SPAN,
        oxc_ast::ast::VariableDeclarationKind::Const,
        ast.binding_pattern_binding_identifier(SPAN, ast.ident("__lux_bindings")),
        NONE,
        Some(
            ast.member_expression_static(
                SPAN,
                ast.expression_identifier(SPAN, ast.ident("__lux_render_result")),
                ast.identifier_name(SPAN, ast.ident("bindings")),
                false,
            )
            .into(),
        ),
        false,
    );
    let actions_decl = ast.variable_declarator(
        SPAN,
        oxc_ast::ast::VariableDeclarationKind::Const,
        ast.binding_pattern_binding_identifier(SPAN, ast.ident("__lux_actions")),
        NONE,
        Some(
            ast.member_expression_static(
                SPAN,
                ast.expression_identifier(SPAN, ast.ident("__lux_render_result")),
                ast.identifier_name(SPAN, ast.ident("actions")),
                false,
            )
            .into(),
        ),
        false,
    );
    let transitions_decl = ast.variable_declarator(
        SPAN,
        oxc_ast::ast::VariableDeclarationKind::Const,
        ast.binding_pattern_binding_identifier(SPAN, ast.ident("__lux_transitions")),
        NONE,
        Some(
            ast.member_expression_static(
                SPAN,
                ast.expression_identifier(SPAN, ast.ident("__lux_render_result")),
                ast.identifier_name(SPAN, ast.ident("transitions")),
                false,
            )
            .into(),
        ),
        false,
    );
    let animations_decl = ast.variable_declarator(
        SPAN,
        oxc_ast::ast::VariableDeclarationKind::Const,
        ast.binding_pattern_binding_identifier(SPAN, ast.ident("__lux_animations")),
        NONE,
        Some(
            ast.member_expression_static(
                SPAN,
                ast.expression_identifier(SPAN, ast.ident("__lux_render_result")),
                ast.identifier_name(SPAN, ast.ident("animations")),
                false,
            )
            .into(),
        ),
        false,
    );
    let render_result_decl = ast.variable_declarator(
        SPAN,
        oxc_ast::ast::VariableDeclarationKind::Const,
        ast.binding_pattern_binding_identifier(SPAN, ast.ident("__lux_render_result")),
        NONE,
        Some(
            ast.expression_call(
                SPAN,
                ast.expression_identifier(SPAN, ast.ident(LUX_END_RENDER)),
                NONE,
                ast.vec1(
                    ast.expression_identifier(SPAN, ast.ident("__lux_render_state"))
                        .into(),
                ),
                false,
            ),
        ),
        false,
    );
    let mount_call = ast.expression_call(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident(LUX_MOUNT_HTML)),
        NONE,
        ast.vec_from_array([
            anchor_ident.clone_in(ast.allocator).into(),
            ast.expression_identifier(SPAN, ast.ident("__lux_html"))
                .into(),
        ]),
        false,
    );
    let mount_head_call = head_html_decl.as_ref().map(|_| {
        ast.expression_call(
            SPAN,
            ast.expression_identifier(SPAN, ast.ident(LUX_MOUNT_HEAD)),
            NONE,
            ast.vec_from_array([
                anchor_ident.clone_in(ast.allocator).into(),
                ast.expression_identifier(SPAN, ast.ident("__lux_head_html"))
                    .into(),
            ]),
            false,
        )
    });
    let cleanup_call = ast.expression_call(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident(LUX_CLEANUP_MOUNT)),
        NONE,
        ast.vec1(anchor_ident.clone_in(ast.allocator).into()),
        false,
    );
    let cleanup_return_call = ast.expression_call(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident(LUX_CLEANUP_MOUNT)),
        NONE,
        ast.vec1(anchor_ident.clone_in(ast.allocator).into()),
        false,
    );
    let cleanup_function = ast.expression_function(
        SPAN,
        FunctionType::FunctionExpression,
        None,
        false,
        false,
        false,
        NONE,
        NONE,
        ast.alloc_formal_parameters(SPAN, FormalParameterKind::FormalParameter, ast.vec(), NONE),
        NONE,
        Some(ast.alloc_function_body(
            SPAN,
            ast.vec(),
            ast.vec1(ast.statement_return(SPAN, Some(cleanup_return_call))),
        )),
    );
    let mount_events_call = ast.expression_call(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident(LUX_MOUNT_EVENTS)),
        NONE,
        ast.vec_from_array([
            anchor_ident.clone_in(ast.allocator).into(),
            ast.expression_identifier(SPAN, ast.ident("__lux_events"))
                .into(),
        ]),
        false,
    );
    let mount_bindings_call = ast.expression_call(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident(LUX_MOUNT_BINDINGS)),
        NONE,
        ast.vec_from_array([
            anchor_ident.clone_in(ast.allocator).into(),
            ast.expression_identifier(SPAN, ast.ident("__lux_bindings"))
                .into(),
        ]),
        false,
    );
    let mount_actions_call = ast.expression_call(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident(LUX_MOUNT_ACTIONS)),
        NONE,
        ast.vec_from_array([
            anchor_ident.clone_in(ast.allocator).into(),
            ast.expression_identifier(SPAN, ast.ident("__lux_actions"))
                .into(),
        ]),
        false,
    );
    let mount_transitions_call = ast.expression_call(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident(LUX_MOUNT_TRANSITIONS)),
        NONE,
        ast.vec_from_array([
            anchor_ident.clone_in(ast.allocator).into(),
            ast.expression_identifier(SPAN, ast.ident("__lux_transitions"))
                .into(),
        ]),
        false,
    );
    let mount_animations_call = ast.expression_call(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident(LUX_MOUNT_ANIMATIONS)),
        NONE,
        ast.vec_from_array([
            anchor_ident.clone_in(ast.allocator).into(),
            ast.expression_identifier(SPAN, ast.ident("__lux_animations"))
                .into(),
        ]),
        false,
    );
    let mut mount_sequence = ast.vec_with_capacity(if mount_head_call.is_some() { 8 } else { 7 });
    mount_sequence.push(cleanup_call);
    mount_sequence.push(mount_call);
    if let Some(mount_head_call) = mount_head_call {
        mount_sequence.push(mount_head_call);
    }
    mount_sequence.push(mount_events_call);
    mount_sequence.push(mount_bindings_call);
    mount_sequence.push(mount_actions_call);
    mount_sequence.push(mount_transitions_call);
    mount_sequence.push(mount_animations_call);
    mount_sequence.push(cleanup_function);
    let mount_result = ast.expression_sequence(SPAN, mount_sequence);
    let return_expr = ast.expression_conditional(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident("__lux_mount_mode")),
        mount_result,
        ast.expression_identifier(SPAN, ast.ident("__lux_html")),
    );

    let mut statements = ast.vec_with_capacity(render_setup_statements.len() + 6);
    statements.push(
        ast.declaration_variable(
            SPAN,
            oxc_ast::ast::VariableDeclarationKind::Const,
            ast.vec1(mount_mode_decl),
            false,
        )
        .into(),
    );
    statements.push(
        ast.declaration_variable(
            SPAN,
            oxc_ast::ast::VariableDeclarationKind::Const,
            ast.vec1(props_decl),
            false,
        )
        .into(),
    );
    statements.push(ast.statement_expression(SPAN, init_self));
    statements.extend(render_setup_statements);
    statements.push(
        ast.declaration_variable(
            SPAN,
            oxc_ast::ast::VariableDeclarationKind::Const,
            ast.vec1(render_state_decl),
            false,
        )
        .into(),
    );
    statements.push(
        ast.declaration_variable(
            SPAN,
            oxc_ast::ast::VariableDeclarationKind::Const,
            ast.vec1(html_decl),
            false,
        )
        .into(),
    );
    if let Some(head_html_decl) = head_html_decl {
        statements.push(
            ast.declaration_variable(
                SPAN,
                oxc_ast::ast::VariableDeclarationKind::Const,
                ast.vec1(head_html_decl),
                false,
            )
            .into(),
        );
    }
    statements.push(
        ast.declaration_variable(
            SPAN,
            oxc_ast::ast::VariableDeclarationKind::Const,
            ast.vec1(render_result_decl),
            false,
        )
        .into(),
    );
    statements.push(
        ast.declaration_variable(
            SPAN,
            oxc_ast::ast::VariableDeclarationKind::Const,
            ast.vec1(events_decl),
            false,
        )
        .into(),
    );
    statements.push(
        ast.declaration_variable(
            SPAN,
            oxc_ast::ast::VariableDeclarationKind::Const,
            ast.vec1(bindings_decl),
            false,
        )
        .into(),
    );
    statements.push(
        ast.declaration_variable(
            SPAN,
            oxc_ast::ast::VariableDeclarationKind::Const,
            ast.vec1(actions_decl),
            false,
        )
        .into(),
    );
    statements.push(
        ast.declaration_variable(
            SPAN,
            oxc_ast::ast::VariableDeclarationKind::Const,
            ast.vec1(transitions_decl),
            false,
        )
        .into(),
    );
    statements.push(
        ast.declaration_variable(
            SPAN,
            oxc_ast::ast::VariableDeclarationKind::Const,
            ast.vec1(animations_decl),
            false,
        )
        .into(),
    );
    statements.push(ast.statement_return(SPAN, Some(return_expr)));

    let function_body = ast.alloc_function_body(SPAN, ast.vec(), statements);
    let function_expression = ast.expression_function(
        SPAN,
        FunctionType::FunctionExpression,
        Some(ast.binding_identifier(SPAN, ast.ident("__lux_component"))),
        false,
        false,
        false,
        NONE,
        NONE,
        params,
        NONE,
        Some(function_body),
    );

    ast.module_declaration_export_default_declaration(
        SPAN,
        ExportDefaultDeclarationKind::from(function_expression),
    )
    .into()
}

fn export_specifier<'a>(
    ast: AstBuilder<'a>,
    local: &str,
    exported: &str,
) -> oxc_ast::ast::ExportSpecifier<'a> {
    ast.export_specifier(
        SPAN,
        ast.module_export_name_identifier_reference(SPAN, ast.ident(local)),
        ast.module_export_name_identifier_name(SPAN, ast.ident(exported)),
        ImportOrExportKind::Value,
    )
}

fn named_property<'a>(
    ast: AstBuilder<'a>,
    key: &str,
    value_ident: &str,
) -> oxc_ast::ast::ObjectPropertyKind<'a> {
    ast.object_property_kind_object_property(
        SPAN,
        PropertyKind::Init,
        ast.property_key_static_identifier(SPAN, ast.ident(key)),
        ast.expression_identifier(SPAN, ast.ident(value_ident)),
        false,
        false,
        false,
    )
}

fn function_property<'a>(
    ast: AstBuilder<'a>,
    key: &str,
    render_expression: Expression<'a>,
    render_setup_statements: oxc_allocator::Vec<'a, Statement<'a>>,
) -> oxc_ast::ast::ObjectPropertyKind<'a> {
    let props_pattern = ast.binding_pattern_assignment_pattern(
        SPAN,
        ast.binding_pattern_binding_identifier(SPAN, ast.ident("_props")),
        ast.expression_object(SPAN, ast.vec()),
    );
    let params = ast.alloc_formal_parameters(
        SPAN,
        FormalParameterKind::FormalParameter,
        ast.vec1(ast.formal_parameter(
            SPAN,
            ast.vec(),
            props_pattern,
            NONE,
            NONE,
            false,
            None,
            false,
            false,
        )),
        NONE,
    );

    let self_member = ast.member_expression_static(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident("_props")),
        ast.identifier_name(SPAN, ast.ident("__lux_self")),
        false,
    );
    let self_missing = ast.expression_binary(
        SPAN,
        self_member.clone_in(ast.allocator).into(),
        BinaryOperator::Equality,
        ast.expression_null_literal(SPAN),
    );
    let self_assign = ast.expression_assignment(
        SPAN,
        AssignmentOperator::Assign,
        self_member.into(),
        ast.expression_identifier(SPAN, ast.ident("__lux_render")),
    );
    let init_self = ast.expression_logical(SPAN, self_missing, LogicalOperator::And, self_assign);
    let mut statements = ast.vec_with_capacity(render_setup_statements.len() + 2);
    statements.push(ast.statement_expression(SPAN, init_self));
    statements.extend(render_setup_statements);
    statements.push(ast.statement_return(SPAN, Some(render_expression)));

    let function_body = ast.alloc_function_body(SPAN, ast.vec(), statements);

    let function_expression = ast.expression_function(
        SPAN,
        FunctionType::FunctionExpression,
        Some(ast.binding_identifier(SPAN, ast.ident("__lux_render"))),
        false,
        false,
        false,
        NONE,
        NONE,
        params,
        NONE,
        Some(function_body),
    );

    ast.object_property_kind_object_property(
        SPAN,
        PropertyKind::Init,
        ast.property_key_static_identifier(SPAN, ast.ident(key)),
        function_expression,
        false,
        false,
        false,
    )
}
