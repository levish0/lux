use lux_ast::template::root::Fragment;
use oxc_allocator::CloneIn;
use oxc_ast::{
    ast::{
        BindingPattern, Expression, FormalParameterKind, FunctionType, Statement,
        VariableDeclarationKind,
    },
    AstBuilder, NONE,
};
use oxc_span::SPAN;

use super::render_fragment_expression;
use super::scope::RuntimeScope;

pub(super) fn call_static_method<'a>(
    ast: AstBuilder<'a>,
    object: Expression<'a>,
    method: &str,
    arguments: oxc_allocator::Vec<'a, oxc_ast::ast::Argument<'a>>,
) -> Expression<'a> {
    let callee = ast.member_expression_static(
        SPAN,
        object,
        ast.identifier_name(SPAN, ast.ident(method)),
        false,
    );
    ast.expression_call(SPAN, callee.into(), NONE, arguments, false)
}

pub(super) fn stringify_expression<'a>(
    ast: AstBuilder<'a>,
    expression: Expression<'a>,
) -> Expression<'a> {
    ast.expression_call(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident("__lux_stringify")),
        NONE,
        ast.vec1(expression.into()),
        false,
    )
}

pub(super) fn escape_html_expression<'a>(
    ast: AstBuilder<'a>,
    value: Expression<'a>,
) -> Expression<'a> {
    ast.expression_call(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident("__lux_escape")),
        NONE,
        ast.vec1(value.into()),
        false,
    )
}

pub(super) fn escape_attr_expression<'a>(
    ast: AstBuilder<'a>,
    value: Expression<'a>,
) -> Expression<'a> {
    ast.expression_call(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident("__lux_escape_attr")),
        NONE,
        ast.vec1(value.into()),
        false,
    )
}

pub(super) fn bind_pattern_value_expression<'a>(
    ast: AstBuilder<'a>,
    pattern: Option<&BindingPattern<'_>>,
    value_ident_name: &str,
    fragment: &Fragment<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let Some(pattern) = pattern else {
        return render_fragment_expression(ast, fragment, scope);
    };

    let bound_scope = scope.with_binding_pattern(pattern);
    let body_expr = render_fragment_expression(ast, fragment, &bound_scope);
    let params = ast.alloc_formal_parameters(
        SPAN,
        FormalParameterKind::FormalParameter,
        ast.vec1(ast.formal_parameter(
            SPAN,
            ast.vec(),
            pattern.clone_in(ast.allocator),
            NONE,
            NONE,
            false,
            None,
            false,
            false,
        )),
        NONE,
    );
    let body = ast.alloc_function_body(
        SPAN,
        ast.vec(),
        ast.vec1(ast.statement_return(SPAN, Some(body_expr))),
    );
    let binder = ast.expression_function(
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
    ast.expression_call(
        SPAN,
        binder,
        NONE,
        ast.vec1(
            ast.expression_identifier(SPAN, ast.ident(value_ident_name))
                .into(),
        ),
        false,
    )
}

pub(super) fn call_iife<'a>(
    ast: AstBuilder<'a>,
    statements: oxc_allocator::Vec<'a, Statement<'a>>,
) -> Expression<'a> {
    let params =
        ast.alloc_formal_parameters(SPAN, FormalParameterKind::FormalParameter, ast.vec(), NONE);
    let body = ast.alloc_function_body(SPAN, ast.vec(), statements);
    let function = ast.expression_function(
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
    ast.expression_call(SPAN, function, NONE, ast.vec(), false)
}

pub(super) fn const_statement<'a>(
    ast: AstBuilder<'a>,
    name: &str,
    init: Expression<'a>,
) -> Statement<'a> {
    let declarator = ast.variable_declarator(
        SPAN,
        VariableDeclarationKind::Const,
        ast.binding_pattern_binding_identifier(SPAN, ast.ident(name)),
        NONE,
        Some(init),
        false,
    );
    ast.declaration_variable(
        SPAN,
        VariableDeclarationKind::Const,
        ast.vec1(declarator),
        false,
    )
    .into()
}

pub(super) fn string_expr<'a>(ast: AstBuilder<'a>, value: &str) -> Expression<'a> {
    ast.expression_string_literal(SPAN, ast.atom(value), None)
}

pub(super) fn join_chunks_expression<'a>(
    ast: AstBuilder<'a>,
    chunks: oxc_allocator::Vec<'a, Expression<'a>>,
) -> Expression<'a> {
    let mut items = ast.vec_with_capacity(chunks.len());
    for chunk in chunks {
        items.push(chunk.into());
    }
    call_static_method(
        ast,
        ast.expression_array(SPAN, items),
        "join",
        ast.vec1(string_expr(ast, "").into()),
    )
}
