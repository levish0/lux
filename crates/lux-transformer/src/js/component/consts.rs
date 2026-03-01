use oxc_allocator::CloneIn;
use oxc_allocator::Vec as ArenaVec;
use oxc_ast::{
    ast::{
        BinaryOperator, Expression, FormalParameterKind, FunctionType, RegExp, RegExpFlags,
        RegExpPattern, Statement, VariableDeclarationKind,
    },
    AstBuilder, NONE,
};
use oxc_span::SPAN;

pub(super) const LUX_TEMPLATE: &str = "__lux_template";
pub(super) const LUX_CSS: &str = "__lux_css";
pub(super) const LUX_CSS_HASH: &str = "__lux_css_hash";
pub(super) const LUX_CSS_SCOPE: &str = "__lux_css_scope";
pub(super) const LUX_HAS_DYNAMIC: &str = "__lux_has_dynamic";
pub(super) const LUX_STRINGIFY: &str = "__lux_stringify";
pub(super) const LUX_ESCAPE: &str = "__lux_escape";
pub(super) const LUX_ESCAPE_ATTR: &str = "__lux_escape_attr";

pub(super) fn push_const<'a>(
    ast: AstBuilder<'a>,
    body: &mut ArenaVec<'a, Statement<'a>>,
    name: &str,
    init: Expression<'a>,
) {
    let declarator = ast.variable_declarator(
        SPAN,
        VariableDeclarationKind::Const,
        ast.binding_pattern_binding_identifier(SPAN, ast.ident(name)),
        NONE,
        Some(init),
        false,
    );
    let declaration = ast.declaration_variable(
        SPAN,
        VariableDeclarationKind::Const,
        ast.vec1(declarator),
        false,
    );
    body.push(declaration.into());
}

pub(super) fn optional_string_expr<'a>(ast: AstBuilder<'a>, value: Option<&str>) -> Expression<'a> {
    value.map_or_else(
        || ast.expression_null_literal(SPAN),
        |value| ast.expression_string_literal(SPAN, ast.atom(value), None),
    )
}

pub(super) fn push_runtime_helpers<'a>(
    ast: AstBuilder<'a>,
    body: &mut ArenaVec<'a, Statement<'a>>,
) {
    push_const(
        ast,
        body,
        LUX_STRINGIFY,
        build_stringify_helper_expression(ast),
    );
    push_const(ast, body, LUX_ESCAPE, build_escape_helper_expression(ast));
    push_const(
        ast,
        body,
        LUX_ESCAPE_ATTR,
        build_escape_attr_helper_expression(ast),
    );
}

fn build_stringify_helper_expression<'a>(ast: AstBuilder<'a>) -> Expression<'a> {
    let value_ident = ast.expression_identifier(SPAN, ast.ident("value"));

    let type_check = ast.expression_binary(
        SPAN,
        ast.expression_unary(
            SPAN,
            oxc_ast::ast::UnaryOperator::Typeof,
            value_ident.clone_in(ast.allocator),
        ),
        BinaryOperator::StrictEquality,
        ast.expression_string_literal(SPAN, ast.atom("string"), None),
    );
    let nullish_check = ast.expression_binary(
        SPAN,
        value_ident.clone_in(ast.allocator),
        BinaryOperator::Equality,
        ast.expression_null_literal(SPAN),
    );
    let value_plus_empty = ast.expression_binary(
        SPAN,
        value_ident,
        BinaryOperator::Addition,
        ast.expression_string_literal(SPAN, ast.atom(""), None),
    );

    let result = ast.expression_conditional(
        SPAN,
        type_check,
        ast.expression_identifier(SPAN, ast.ident("value")),
        ast.expression_conditional(
            SPAN,
            nullish_check,
            ast.expression_string_literal(SPAN, ast.atom(""), None),
            value_plus_empty,
        ),
    );

    single_param_function_expression(ast, "value", result)
}

fn build_escape_helper_expression<'a>(ast: AstBuilder<'a>) -> Expression<'a> {
    let stringify_call = ast.expression_call(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident(LUX_STRINGIFY)),
        NONE,
        ast.vec1(ast.expression_identifier(SPAN, ast.ident("value")).into()),
        false,
    );
    let escaped = call_method(
        ast,
        stringify_call,
        "replace",
        ast.vec_from_array([
            ast.expression_reg_exp_literal(
                SPAN,
                RegExp {
                    pattern: RegExpPattern {
                        text: ast.atom("[&<>]"),
                        pattern: None,
                    },
                    flags: RegExpFlags::G,
                },
                Some(ast.atom("/[&<>]/g")),
            )
            .into(),
            build_html_escape_replacer(ast).into(),
        ]),
    );

    single_param_function_expression(ast, "value", escaped)
}

fn build_escape_attr_helper_expression<'a>(ast: AstBuilder<'a>) -> Expression<'a> {
    let escaped = call_method(
        ast,
        ast.expression_call(
            SPAN,
            ast.expression_identifier(SPAN, ast.ident(LUX_STRINGIFY)),
            NONE,
            ast.vec1(ast.expression_identifier(SPAN, ast.ident("value")).into()),
            false,
        ),
        "replace",
        ast.vec_from_array([
            ast.expression_reg_exp_literal(
                SPAN,
                RegExp {
                    pattern: RegExpPattern {
                        text: ast.atom("[&<>\"']"),
                        pattern: None,
                    },
                    flags: RegExpFlags::G,
                },
                Some(ast.atom("/[&<>\"']/g")),
            )
            .into(),
            build_attr_escape_replacer(ast).into(),
        ]),
    );

    single_param_function_expression(ast, "value", escaped)
}

fn build_html_escape_replacer<'a>(ast: AstBuilder<'a>) -> Expression<'a> {
    let ch_ident = ast.expression_identifier(SPAN, ast.ident("ch"));
    let amp_check = ast.expression_binary(
        SPAN,
        ch_ident.clone_in(ast.allocator),
        BinaryOperator::StrictEquality,
        ast.expression_string_literal(SPAN, ast.atom("&"), None),
    );
    let lt_check = ast.expression_binary(
        SPAN,
        ch_ident.clone_in(ast.allocator),
        BinaryOperator::StrictEquality,
        ast.expression_string_literal(SPAN, ast.atom("<"), None),
    );
    let result = ast.expression_conditional(
        SPAN,
        amp_check,
        ast.expression_string_literal(SPAN, ast.atom("&amp;"), None),
        ast.expression_conditional(
            SPAN,
            lt_check,
            ast.expression_string_literal(SPAN, ast.atom("&lt;"), None),
            ast.expression_string_literal(SPAN, ast.atom("&gt;"), None),
        ),
    );

    single_param_function_expression(ast, "ch", result)
}

fn build_attr_escape_replacer<'a>(ast: AstBuilder<'a>) -> Expression<'a> {
    let ch_ident = ast.expression_identifier(SPAN, ast.ident("ch"));
    let amp_check = ast.expression_binary(
        SPAN,
        ch_ident.clone_in(ast.allocator),
        BinaryOperator::StrictEquality,
        ast.expression_string_literal(SPAN, ast.atom("&"), None),
    );
    let lt_check = ast.expression_binary(
        SPAN,
        ch_ident.clone_in(ast.allocator),
        BinaryOperator::StrictEquality,
        ast.expression_string_literal(SPAN, ast.atom("<"), None),
    );
    let gt_check = ast.expression_binary(
        SPAN,
        ch_ident.clone_in(ast.allocator),
        BinaryOperator::StrictEquality,
        ast.expression_string_literal(SPAN, ast.atom(">"), None),
    );
    let quote_check = ast.expression_binary(
        SPAN,
        ch_ident.clone_in(ast.allocator),
        BinaryOperator::StrictEquality,
        ast.expression_string_literal(SPAN, ast.atom("\""), None),
    );

    let result = ast.expression_conditional(
        SPAN,
        amp_check,
        ast.expression_string_literal(SPAN, ast.atom("&amp;"), None),
        ast.expression_conditional(
            SPAN,
            lt_check,
            ast.expression_string_literal(SPAN, ast.atom("&lt;"), None),
            ast.expression_conditional(
                SPAN,
                gt_check,
                ast.expression_string_literal(SPAN, ast.atom("&gt;"), None),
                ast.expression_conditional(
                    SPAN,
                    quote_check,
                    ast.expression_string_literal(SPAN, ast.atom("&quot;"), None),
                    ast.expression_string_literal(SPAN, ast.atom("&#39;"), None),
                ),
            ),
        ),
    );

    single_param_function_expression(ast, "ch", result)
}

fn single_param_function_expression<'a>(
    ast: AstBuilder<'a>,
    param_name: &str,
    body_expression: Expression<'a>,
) -> Expression<'a> {
    let params = ast.alloc_formal_parameters(
        SPAN,
        FormalParameterKind::FormalParameter,
        ast.vec1(ast.formal_parameter(
            SPAN,
            ast.vec(),
            ast.binding_pattern_binding_identifier(SPAN, ast.ident(param_name)),
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
        ast.vec1(ast.statement_return(SPAN, Some(body_expression))),
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

fn call_method<'a>(
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
