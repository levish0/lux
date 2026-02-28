use lux_ast::template::block::{AwaitBlock, EachBlock, IfBlock, SnippetBlock};
use oxc_allocator::CloneIn;
use oxc_ast::{
    AstBuilder, NONE,
    ast::{
        AssignmentOperator, BinaryOperator, Expression, FormalParameterKind, FunctionType,
        LogicalOperator, UnaryOperator,
    },
};
use oxc_span::SPAN;

use super::expr::{
    bind_pattern_value_expression, call_iife, call_static_method, const_statement, string_expr,
};
use super::render_fragment_expression;
use super::scope::{RuntimeScope, resolve_expression};

pub(super) fn render_if_block_expression<'a>(
    ast: AstBuilder<'a>,
    block: &IfBlock<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let test = resolve_expression(ast, block.test.clone_in(ast.allocator), scope);
    let consequent = render_fragment_expression(ast, &block.consequent, scope);
    let alternate = block.alternate.as_ref().map_or_else(
        || string_expr(ast, ""),
        |alternate| render_fragment_expression(ast, alternate, scope),
    );

    ast.expression_conditional(SPAN, test, consequent, alternate)
}

pub(super) fn render_each_block_expression<'a>(
    ast: AstBuilder<'a>,
    block: &EachBlock<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let source = resolve_expression(ast, block.expression.clone_in(ast.allocator), scope);
    let iterable = ast.expression_logical(
        SPAN,
        source,
        LogicalOperator::Coalesce,
        ast.expression_array(SPAN, ast.vec()),
    );
    let from_call = call_static_method(
        ast,
        ast.expression_identifier(SPAN, ast.ident("Array")),
        "from",
        ast.vec1(iterable.into()),
    );

    let mut params_items = ast.vec_with_capacity(if block.index.is_some() { 2 } else { 1 });
    let context_pattern = block.context.as_ref().map_or_else(
        || ast.binding_pattern_binding_identifier(SPAN, ast.ident("__item")),
        |pattern| pattern.clone_in(ast.allocator),
    );
    params_items.push(ast.formal_parameter(
        SPAN,
        ast.vec(),
        context_pattern,
        NONE,
        NONE,
        false,
        None,
        false,
        false,
    ));
    if let Some(index) = block.index {
        params_items.push(ast.formal_parameter(
            SPAN,
            ast.vec(),
            ast.binding_pattern_binding_identifier(SPAN, ast.ident(index)),
            NONE,
            NONE,
            false,
            None,
            false,
            false,
        ));
    }

    let params = ast.alloc_formal_parameters(
        SPAN,
        FormalParameterKind::FormalParameter,
        params_items,
        NONE,
    );
    let mut body_scope = scope.clone();
    if let Some(context) = &block.context {
        body_scope = body_scope.with_binding_pattern(context);
    }
    if let Some(index) = block.index {
        body_scope = body_scope.with_name(index);
    }

    let body_expr = render_fragment_expression(ast, &block.body, &body_scope);
    let body = ast.alloc_function_body(
        SPAN,
        ast.vec(),
        ast.vec1(ast.statement_return(SPAN, Some(body_expr))),
    );
    let callback = ast.expression_function(
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

    let mapped = call_static_method(
        ast,
        from_call.clone_in(ast.allocator),
        "map",
        ast.vec1(callback.into()),
    );
    let joined = call_static_method(ast, mapped, "join", ast.vec1(string_expr(ast, "").into()));

    if let Some(fallback) = &block.fallback {
        let fallback_expr = render_fragment_expression(ast, fallback, scope);
        let len_expr = ast.member_expression_static(
            SPAN,
            from_call,
            ast.identifier_name(SPAN, ast.ident("length")),
            false,
        );
        let has_items = ast.expression_binary(
            SPAN,
            len_expr.into(),
            BinaryOperator::GreaterThan,
            ast.expression_numeric_literal(SPAN, 0.0, None, oxc_ast::ast::NumberBase::Decimal),
        );
        ast.expression_conditional(SPAN, has_items, joined, fallback_expr)
    } else {
        joined
    }
}

pub(super) fn render_await_block_expression<'a>(
    ast: AstBuilder<'a>,
    block: &AwaitBlock<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let awaited_expression = resolve_expression(ast, block.expression.clone_in(ast.allocator), scope);

    let pending_expr = block.pending.as_ref().map_or_else(
        || string_expr(ast, ""),
        |pending| render_fragment_expression(ast, pending, scope),
    );

    let then_expr = block.then.as_ref().map_or_else(
        || string_expr(ast, ""),
        |then_fragment| {
            bind_pattern_value_expression(
                ast,
                block.value.as_ref(),
                "__lux_await_value",
                then_fragment,
                scope,
            )
        },
    );

    let await_value_ident = ast.expression_identifier(SPAN, ast.ident("__lux_await_value"));
    let has_then = ast.member_expression_static(
        SPAN,
        await_value_ident.clone_in(ast.allocator),
        ast.identifier_name(SPAN, ast.ident("then")),
        false,
    );
    let promise_like = ast.expression_logical(
        SPAN,
        await_value_ident.clone_in(ast.allocator),
        LogicalOperator::And,
        ast.expression_binary(
            SPAN,
            ast.expression_unary(SPAN, UnaryOperator::Typeof, has_then.into()),
            BinaryOperator::StrictEquality,
            string_expr(ast, "function"),
        ),
    );

    let try_result = ast.expression_conditional(SPAN, promise_like, pending_expr, then_expr);

    let mut body_statements = ast.vec();
    if let Some(catch_fragment) = &block.catch {
        let try_block = ast.alloc_block_statement(
            SPAN,
            ast.vec_from_array([
                const_statement(ast, "__lux_await_value", awaited_expression),
                ast.statement_return(SPAN, Some(try_result)),
            ]),
        );
        let catch_pattern = block.error.as_ref().map_or_else(
            || ast.binding_pattern_binding_identifier(SPAN, ast.ident("__lux_await_error")),
            |pattern| pattern.clone_in(ast.allocator),
        );
        let catch_scope = scope.with_binding_pattern(&catch_pattern);
        let catch_expr = render_fragment_expression(ast, catch_fragment, &catch_scope);
        let catch_clause = ast.catch_clause(
            SPAN,
            Some(ast.catch_parameter(SPAN, catch_pattern, NONE)),
            ast.alloc_block_statement(SPAN, ast.vec1(ast.statement_return(SPAN, Some(catch_expr)))),
        );
        body_statements.push(ast.statement_try(SPAN, try_block, Some(catch_clause), NONE));
    } else {
        body_statements.push(const_statement(ast, "__lux_await_value", awaited_expression));
        body_statements.push(ast.statement_return(SPAN, Some(try_result)));
    }

    call_iife(ast, body_statements)
}

pub(super) fn render_snippet_block_declaration<'a>(
    ast: AstBuilder<'a>,
    block: &SnippetBlock<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let name = block.expression.name.as_str();

    let mut snippet_scope = scope.with_name(name);
    for parameter in &block.parameters {
        snippet_scope = snippet_scope.with_binding_pattern(parameter);
    }

    let mut params_items = ast.vec_with_capacity(block.parameters.len());
    for parameter in &block.parameters {
        params_items.push(ast.formal_parameter(
            SPAN,
            ast.vec(),
            parameter.clone_in(ast.allocator),
            NONE,
            NONE,
            false,
            None,
            false,
            false,
        ));
    }

    let params = ast.alloc_formal_parameters(
        SPAN,
        FormalParameterKind::FormalParameter,
        params_items,
        NONE,
    );
    let body_expr = render_fragment_expression(ast, &block.body, &snippet_scope);
    let body = ast.alloc_function_body(
        SPAN,
        ast.vec(),
        ast.vec1(ast.statement_return(SPAN, Some(body_expr))),
    );
    let snippet_function = ast.expression_function(
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

    let snippet_target = ast.member_expression_static(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident("_props")),
        ast.identifier_name(SPAN, ast.ident(name)),
        false,
    );
    let assignment = ast.expression_assignment(
        SPAN,
        AssignmentOperator::Assign,
        snippet_target.into(),
        snippet_function,
    );

    ast.expression_sequence(SPAN, ast.vec_from_array([assignment, string_expr(ast, "")]))
}
