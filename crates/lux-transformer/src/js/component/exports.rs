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

use super::consts::{LUX_CSS, LUX_CSS_HASH, LUX_CSS_SCOPE, LUX_HAS_DYNAMIC, LUX_TEMPLATE};

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
) -> Statement<'a> {
    let mut properties = ast.vec_with_capacity(6);
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

    let object_expression = ast.expression_object(SPAN, properties);
    ast.module_declaration_export_default_declaration(
        SPAN,
        ExportDefaultDeclarationKind::from(object_expression),
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
