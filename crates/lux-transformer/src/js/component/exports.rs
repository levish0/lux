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
    LUX_RENDER_COMPONENT, LUX_TEMPLATE,
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

pub(super) fn default_export_statements<'a>(
    ast: AstBuilder<'a>,
    render_expression: Expression<'a>,
    render_setup_statements: oxc_allocator::Vec<'a, Statement<'a>>,
    head_expression: Option<Expression<'a>>,
    _head_setup_statements: Option<oxc_allocator::Vec<'a, Statement<'a>>>,
) -> oxc_allocator::Vec<'a, Statement<'a>> {
    let mut statements = ast.vec_with_capacity(2);
    statements.push(server_component_export_statement(
        ast,
        render_expression,
        render_setup_statements,
        head_expression,
    ));
    statements.push(server_component_metadata_statement(ast));
    statements
}

fn server_component_export_statement<'a>(
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
                ast.binding_pattern_binding_identifier(SPAN, ast.ident("$$renderer")),
                NONE,
                NONE,
                false,
                None,
                false,
                false,
            ),
            default_props_formal_parameter(ast),
        ]),
        NONE,
    );

    let head_html_decl = ast.variable_declarator(
        SPAN,
        oxc_ast::ast::VariableDeclarationKind::Const,
        ast.binding_pattern_binding_identifier(SPAN, ast.ident("__lux_head_html")),
        NONE,
        Some(head_expression.unwrap_or_else(|| {
            ast.expression_string_literal(SPAN, ast.atom(""), None)
        })),
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
    let combined_head_decl = ast.variable_declarator(
        SPAN,
        oxc_ast::ast::VariableDeclarationKind::Const,
        ast.binding_pattern_binding_identifier(SPAN, ast.ident("__lux_combined_head")),
        NONE,
        Some(ast.expression_binary(
            SPAN,
            ast.expression_identifier(SPAN, ast.ident("__lux_head_html")),
            BinaryOperator::Addition,
            ast.member_expression_static(
                SPAN,
                ast.expression_identifier(SPAN, ast.ident("__lux_render_result")),
                ast.identifier_name(SPAN, ast.ident("head")),
                false,
            )
            .into(),
        )),
        false,
    );

    let mut statements = ast.vec_with_capacity(render_setup_statements.len() + 8);
    statements.push(ast.statement_expression(
        SPAN,
        init_self_expression(ast, "__lux_component"),
    ));
    statements.extend(render_setup_statements);
    statements.push(const_declaration_statement(ast, render_state_declarator(ast)));
    statements.push(const_declaration_statement(ast, head_html_decl));
    statements.push(const_declaration_statement(ast, html_decl));
    statements.push(const_declaration_statement(ast, render_result_declarator(ast)));
    statements.push(const_declaration_statement(ast, combined_head_decl));
    statements.push(server_head_mount_statement(ast));
    statements.push(ast.statement_expression(
        SPAN,
        ast.expression_call(
            SPAN,
            ast.member_expression_static(
                SPAN,
                ast.expression_identifier(SPAN, ast.ident("$$renderer")),
                ast.identifier_name(SPAN, ast.ident("push")),
                false,
            )
            .into(),
            NONE,
            ast.vec1(ast.expression_identifier(SPAN, ast.ident("__lux_html")).into()),
            false,
        ),
    ));

    ast.module_declaration_export_default_declaration(
        SPAN,
        ast.export_default_declaration_kind_function_declaration(
            SPAN,
            FunctionType::FunctionDeclaration,
            Some(ast.binding_identifier(SPAN, ast.ident("__lux_component"))),
            false,
            false,
            false,
            NONE,
            NONE,
            params,
            NONE,
            Some(ast.alloc_function_body(SPAN, ast.vec(), statements)),
        ),
    )
    .into()
}

fn server_component_metadata_statement<'a>(ast: AstBuilder<'a>) -> Statement<'a> {
    let mut properties = ast.vec_with_capacity(7);
    properties.push(named_property(ast, "template", LUX_TEMPLATE));
    properties.push(named_property(ast, "css", LUX_CSS));
    properties.push(named_property(ast, "cssHash", LUX_CSS_HASH));
    properties.push(named_property(ast, "cssScope", LUX_CSS_SCOPE));
    properties.push(named_property(ast, "hasDynamic", LUX_HAS_DYNAMIC));
    properties.push(function_property(ast, "render", "__lux_render", false));
    properties.push(function_property(ast, "head", "__lux_head", true));

    let object_expression = ast.expression_object(SPAN, properties);
    ast.statement_expression(
        SPAN,
        ast.expression_call(
            SPAN,
            ast.member_expression_static(
                SPAN,
                ast.expression_identifier(SPAN, ast.ident("Object")),
                ast.identifier_name(SPAN, ast.ident("assign")),
                false,
            )
            .into(),
            NONE,
            ast.vec_from_array([
                ast.expression_identifier(SPAN, ast.ident("__lux_component"))
                    .into(),
                object_expression.into(),
            ]),
            false,
        ),
    )
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
    function_name: &str,
    return_head: bool,
) -> oxc_ast::ast::ObjectPropertyKind<'a> {
    ast.object_property_kind_object_property(
        SPAN,
        PropertyKind::Init,
        ast.property_key_static_identifier(SPAN, ast.ident(key)),
        compatibility_function_expression(ast, function_name, return_head),
        false,
        false,
        false,
    )
}

fn compatibility_function_expression<'a>(
    ast: AstBuilder<'a>,
    function_name: &str,
    return_head: bool,
) -> Expression<'a> {
    let params = ast.alloc_formal_parameters(
        SPAN,
        FormalParameterKind::FormalParameter,
        ast.vec1(default_props_formal_parameter(ast)),
        NONE,
    );
    let mut statements = ast.vec_with_capacity(if return_head { 4 } else { 1 });
    if return_head {
        let render_component_call = ast.expression_call(
            SPAN,
            ast.expression_identifier(SPAN, ast.ident(LUX_RENDER_COMPONENT)),
            NONE,
            ast.vec_from_array([
                ast.expression_identifier(SPAN, ast.ident("__lux_component"))
                    .into(),
                ast.expression_identifier(SPAN, ast.ident("_props")).into(),
                ast.expression_identifier(SPAN, ast.ident("__lux_render_state"))
                    .into(),
            ]),
            false,
        );
        statements.push(const_declaration_statement(ast, render_state_declarator(ast)));
        statements.push(ast.statement_expression(SPAN, render_component_call));
        statements.push(const_declaration_statement(ast, render_result_declarator(ast)));
        statements.push(ast.statement_return(
            SPAN,
            Some(
                ast.member_expression_static(
                    SPAN,
                    ast.expression_identifier(SPAN, ast.ident("__lux_render_result")),
                    ast.identifier_name(SPAN, ast.ident("head")),
                    false,
                )
                .into(),
            ),
        ));
    } else {
        statements.push(ast.statement_return(
            SPAN,
            Some(
                ast.expression_call(
                    SPAN,
                    ast.expression_identifier(SPAN, ast.ident(LUX_RENDER_COMPONENT)),
                    NONE,
                    ast.vec_from_array([
                        ast.expression_identifier(SPAN, ast.ident("__lux_component"))
                            .into(),
                        ast.expression_identifier(SPAN, ast.ident("_props")).into(),
                    ]),
                    false,
                ),
            ),
        ));
    }

    let function_body = ast.alloc_function_body(SPAN, ast.vec(), statements);
    ast.expression_function(
        SPAN,
        FunctionType::FunctionExpression,
        Some(ast.binding_identifier(SPAN, ast.ident(function_name))),
        false,
        false,
        false,
        NONE,
        NONE,
        params,
        NONE,
        Some(function_body),
    )
}

fn default_props_formal_parameter<'a>(
    ast: AstBuilder<'a>,
) -> oxc_ast::ast::FormalParameter<'a> {
    let props_pattern = ast.binding_pattern_assignment_pattern(
        SPAN,
        ast.binding_pattern_binding_identifier(SPAN, ast.ident("_props")),
        ast.expression_object(SPAN, ast.vec()),
    );
    ast.formal_parameter(
        SPAN,
        ast.vec(),
        props_pattern,
        NONE,
        NONE,
        false,
        None,
        false,
        false,
    )
}

fn init_self_expression<'a>(ast: AstBuilder<'a>, target_ident: &str) -> Expression<'a> {
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
        ast.expression_identifier(SPAN, ast.ident(target_ident)),
    );
    ast.expression_logical(SPAN, self_missing, LogicalOperator::And, self_assign)
}

fn const_declaration_statement<'a>(
    ast: AstBuilder<'a>,
    declarator: oxc_ast::ast::VariableDeclarator<'a>,
) -> Statement<'a> {
    ast.declaration_variable(
        SPAN,
        oxc_ast::ast::VariableDeclarationKind::Const,
        ast.vec1(declarator),
        false,
    )
    .into()
}

fn render_state_declarator<'a>(ast: AstBuilder<'a>) -> oxc_ast::ast::VariableDeclarator<'a> {
    ast.variable_declarator(
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
    )
}

fn render_result_declarator<'a>(ast: AstBuilder<'a>) -> oxc_ast::ast::VariableDeclarator<'a> {
    ast.variable_declarator(
        SPAN,
        oxc_ast::ast::VariableDeclarationKind::Const,
        ast.binding_pattern_binding_identifier(SPAN, ast.ident("__lux_render_result")),
        NONE,
        Some(ast.expression_call(
            SPAN,
            ast.expression_identifier(SPAN, ast.ident(LUX_END_RENDER)),
            NONE,
            ast.vec1(
                ast.expression_identifier(SPAN, ast.ident("__lux_render_state"))
                    .into(),
            ),
            false,
        )),
        false,
    )
}

fn server_head_mount_statement<'a>(ast: AstBuilder<'a>) -> Statement<'a> {
    let push_head_statement = ast.statement_expression(
        SPAN,
        ast.expression_call(
            SPAN,
            ast.member_expression_static(
                SPAN,
                ast.expression_identifier(SPAN, ast.ident("__lux_head_renderer")),
                ast.identifier_name(SPAN, ast.ident("push")),
                false,
            )
            .into(),
            NONE,
            ast.vec1(
                ast.expression_identifier(SPAN, ast.ident("__lux_combined_head"))
                    .into(),
            ),
            false,
        ),
    );
    let head_callback = ast.expression_function(
        SPAN,
        FunctionType::FunctionExpression,
        None,
        false,
        false,
        false,
        NONE,
        NONE,
        ast.alloc_formal_parameters(
            SPAN,
            FormalParameterKind::FormalParameter,
            ast.vec1(ast.formal_parameter(
                SPAN,
                ast.vec(),
                ast.binding_pattern_binding_identifier(
                    SPAN,
                    ast.ident("__lux_head_renderer"),
                ),
                NONE,
                NONE,
                false,
                None,
                false,
                false,
            )),
            NONE,
        ),
        NONE,
        Some(ast.alloc_function_body(
            SPAN,
            ast.vec(),
            ast.vec1(push_head_statement),
        )),
    );
    ast.statement_expression(
        SPAN,
        ast.expression_logical(
            SPAN,
            ast.expression_identifier(SPAN, ast.ident("__lux_combined_head")),
            LogicalOperator::And,
            ast.expression_call(
                SPAN,
                ast.member_expression_static(
                    SPAN,
                    ast.expression_identifier(SPAN, ast.ident("$$renderer")),
                    ast.identifier_name(SPAN, ast.ident("head")),
                    false,
                )
                .into(),
                NONE,
                ast.vec1(head_callback.into()),
                false,
            ),
        ),
    )
}
