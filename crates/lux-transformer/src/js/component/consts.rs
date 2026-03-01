use oxc_allocator::CloneIn;
use oxc_allocator::Vec as ArenaVec;
use oxc_ast::{
    AstBuilder, NONE,
    ast::{
        BinaryOperator, Expression, FormalParameterKind, FunctionType, RegExp, RegExpFlags,
        RegExpPattern, Statement, VariableDeclarationKind,
    },
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
pub(super) const LUX_ATTR: &str = "__lux_attr";
pub(super) const LUX_ATTRIBUTES: &str = "__lux_attributes";
pub(super) const LUX_IS_BOOLEAN_ATTR: &str = "__lux_is_boolean_attr";

const BOOLEAN_ATTRIBUTE_NAMES: &[&str] = &[
    "allowfullscreen",
    "async",
    "autofocus",
    "autoplay",
    "checked",
    "controls",
    "default",
    "defer",
    "disabled",
    "disablepictureinpicture",
    "disableremoteplayback",
    "formnovalidate",
    "indeterminate",
    "inert",
    "ismap",
    "loop",
    "multiple",
    "muted",
    "nomodule",
    "novalidate",
    "open",
    "playsinline",
    "readonly",
    "required",
    "reversed",
    "seamless",
    "selected",
    "webkitdirectory",
];

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
    push_const(
        ast,
        body,
        LUX_IS_BOOLEAN_ATTR,
        build_is_boolean_attr_helper_expression(ast),
    );
    push_const(ast, body, LUX_ATTR, build_attr_helper_expression(ast));
    push_const(
        ast,
        body,
        LUX_ATTRIBUTES,
        build_attributes_helper_expression(ast),
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

fn build_attr_helper_expression<'a>(ast: AstBuilder<'a>) -> Expression<'a> {
    let name_ident = ast.expression_identifier(SPAN, ast.ident("name"));
    let value_ident = ast.expression_identifier(SPAN, ast.ident("value"));
    let is_boolean_ident = ast.expression_identifier(SPAN, ast.ident("is_boolean"));

    let name_string = ast.expression_call(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident(LUX_STRINGIFY)),
        NONE,
        ast.vec1(name_ident.clone_in(ast.allocator).into()),
        false,
    );
    let lower_name = call_method(
        ast,
        name_string.clone_in(ast.allocator),
        "toLowerCase",
        ast.vec(),
    );

    let hidden_is_boolean = ast.expression_logical(
        SPAN,
        ast.expression_binary(
            SPAN,
            lower_name.clone_in(ast.allocator),
            BinaryOperator::StrictEquality,
            ast.expression_string_literal(SPAN, ast.atom("hidden"), None),
        ),
        oxc_ast::ast::LogicalOperator::And,
        ast.expression_binary(
            SPAN,
            value_ident.clone_in(ast.allocator),
            BinaryOperator::StrictInequality,
            ast.expression_string_literal(SPAN, ast.atom("until-found"), None),
        ),
    );
    let effective_boolean = ast.expression_logical(
        SPAN,
        is_boolean_ident,
        oxc_ast::ast::LogicalOperator::Or,
        hidden_is_boolean,
    );

    let normalized_value = ast.expression_conditional(
        SPAN,
        ast.expression_logical(
            SPAN,
            ast.expression_binary(
                SPAN,
                lower_name.clone_in(ast.allocator),
                BinaryOperator::StrictEquality,
                ast.expression_string_literal(SPAN, ast.atom("translate"), None),
            ),
            oxc_ast::ast::LogicalOperator::And,
            ast.expression_binary(
                SPAN,
                value_ident.clone_in(ast.allocator),
                BinaryOperator::StrictEquality,
                ast.expression_boolean_literal(SPAN, true),
            ),
        ),
        ast.expression_string_literal(SPAN, ast.atom("yes"), None),
        ast.expression_conditional(
            SPAN,
            ast.expression_logical(
                SPAN,
                ast.expression_binary(
                    SPAN,
                    lower_name.clone_in(ast.allocator),
                    BinaryOperator::StrictEquality,
                    ast.expression_string_literal(SPAN, ast.atom("translate"), None),
                ),
                oxc_ast::ast::LogicalOperator::And,
                ast.expression_binary(
                    SPAN,
                    value_ident.clone_in(ast.allocator),
                    BinaryOperator::StrictEquality,
                    ast.expression_boolean_literal(SPAN, false),
                ),
            ),
            ast.expression_string_literal(SPAN, ast.atom("no"), None),
            value_ident.clone_in(ast.allocator),
        ),
    );

    let omitted = ast.expression_logical(
        SPAN,
        ast.expression_binary(
            SPAN,
            value_ident.clone_in(ast.allocator),
            BinaryOperator::Equality,
            ast.expression_null_literal(SPAN),
        ),
        oxc_ast::ast::LogicalOperator::Or,
        ast.expression_logical(
            SPAN,
            effective_boolean.clone_in(ast.allocator),
            oxc_ast::ast::LogicalOperator::And,
            ast.expression_unary(
                SPAN,
                oxc_ast::ast::UnaryOperator::LogicalNot,
                value_ident.clone_in(ast.allocator),
            ),
        ),
    );

    let bare_attr = ast.expression_binary(
        SPAN,
        ast.expression_string_literal(SPAN, ast.atom(" "), None),
        BinaryOperator::Addition,
        name_string.clone_in(ast.allocator),
    );
    let value_attr = ast.expression_binary(
        SPAN,
        ast.expression_binary(
            SPAN,
            ast.expression_binary(
                SPAN,
                bare_attr.clone_in(ast.allocator),
                BinaryOperator::Addition,
                ast.expression_string_literal(SPAN, ast.atom("=\""), None),
            ),
            BinaryOperator::Addition,
            ast.expression_call(
                SPAN,
                ast.expression_identifier(SPAN, ast.ident(LUX_ESCAPE_ATTR)),
                NONE,
                ast.vec1(normalized_value.into()),
                false,
            ),
        ),
        BinaryOperator::Addition,
        ast.expression_string_literal(SPAN, ast.atom("\""), None),
    );
    let result = ast.expression_conditional(
        SPAN,
        omitted,
        ast.expression_string_literal(SPAN, ast.atom(""), None),
        ast.expression_conditional(SPAN, effective_boolean, bare_attr, value_attr),
    );

    function_expression_with_params(ast, &["name", "value", "is_boolean"], result)
}

fn build_is_boolean_attr_helper_expression<'a>(ast: AstBuilder<'a>) -> Expression<'a> {
    let mut names = ast.vec_with_capacity(BOOLEAN_ATTRIBUTE_NAMES.len());
    for name in BOOLEAN_ATTRIBUTE_NAMES {
        names.push(
            ast.expression_string_literal(SPAN, ast.atom(*name), None)
                .into(),
        );
    }
    let name_string = ast.expression_call(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident(LUX_STRINGIFY)),
        NONE,
        ast.vec1(ast.expression_identifier(SPAN, ast.ident("name")).into()),
        false,
    );
    let lower_name = call_method(ast, name_string, "toLowerCase", ast.vec());
    let includes = call_method(
        ast,
        ast.expression_array(SPAN, names),
        "includes",
        ast.vec1(lower_name.into()),
    );

    single_param_function_expression(ast, "name", includes)
}

fn build_attributes_helper_expression<'a>(ast: AstBuilder<'a>) -> Expression<'a> {
    let attrs_ident = ast.expression_identifier(SPAN, ast.ident("attrs"));
    let attrs_object = ast.expression_logical(
        SPAN,
        attrs_ident,
        oxc_ast::ast::LogicalOperator::Coalesce,
        ast.expression_object(SPAN, ast.vec()),
    );
    let entries = ast.expression_call(
        SPAN,
        ast.member_expression_static(
            SPAN,
            ast.expression_identifier(SPAN, ast.ident("Object")),
            ast.identifier_name(SPAN, ast.ident("entries")),
            false,
        )
        .into(),
        NONE,
        ast.vec1(attrs_object.into()),
        false,
    );

    let entry_ident = ast.expression_identifier(SPAN, ast.ident("__lux_entry"));
    let key_expr = ast.member_expression_computed(
        SPAN,
        entry_ident.clone_in(ast.allocator),
        ast.expression_numeric_literal(SPAN, 0.0, None, oxc_ast::ast::NumberBase::Decimal),
        false,
    );
    let value_expr = ast.member_expression_computed(
        SPAN,
        entry_ident.clone_in(ast.allocator),
        ast.expression_numeric_literal(SPAN, 1.0, None, oxc_ast::ast::NumberBase::Decimal),
        false,
    );

    let value_is_function = ast.expression_binary(
        SPAN,
        ast.expression_unary(
            SPAN,
            oxc_ast::ast::UnaryOperator::Typeof,
            value_expr.clone_in(ast.allocator).into(),
        ),
        BinaryOperator::StrictEquality,
        ast.expression_string_literal(SPAN, ast.atom("function"), None),
    );
    let key_string = ast.expression_call(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident(LUX_STRINGIFY)),
        NONE,
        ast.vec1(key_expr.into()),
        false,
    );
    let key_starts_with_internal = ast.expression_call(
        SPAN,
        ast.member_expression_static(
            SPAN,
            key_string.clone_in(ast.allocator),
            ast.identifier_name(SPAN, ast.ident("startsWith")),
            false,
        )
        .into(),
        NONE,
        ast.vec1(
            ast.expression_string_literal(SPAN, ast.atom("$$"), None)
                .into(),
        ),
        false,
    );
    let omitted = ast.expression_logical(
        SPAN,
        value_is_function,
        oxc_ast::ast::LogicalOperator::Or,
        key_starts_with_internal,
    );
    let mapped_value = ast.expression_conditional(
        SPAN,
        omitted,
        ast.expression_string_literal(SPAN, ast.atom(""), None),
        ast.expression_call(
            SPAN,
            ast.expression_identifier(SPAN, ast.ident(LUX_ATTR)),
            NONE,
            ast.vec_from_array([
                key_string.clone_in(ast.allocator).into(),
                value_expr.into(),
                ast.expression_call(
                    SPAN,
                    ast.expression_identifier(SPAN, ast.ident(LUX_IS_BOOLEAN_ATTR)),
                    NONE,
                    ast.vec1(key_string.into()),
                    false,
                )
                .into(),
            ]),
            false,
        ),
    );
    let mapper = single_param_function_expression(ast, "__lux_entry", mapped_value);
    let mapped = call_method(ast, entries, "map", ast.vec1(mapper.into()));
    let joined = call_method(
        ast,
        mapped,
        "join",
        ast.vec1(
            ast.expression_string_literal(SPAN, ast.atom(""), None)
                .into(),
        ),
    );

    single_param_function_expression(ast, "attrs", joined)
}

fn single_param_function_expression<'a>(
    ast: AstBuilder<'a>,
    param_name: &str,
    body_expression: Expression<'a>,
) -> Expression<'a> {
    function_expression_with_params(ast, &[param_name], body_expression)
}

fn function_expression_with_params<'a>(
    ast: AstBuilder<'a>,
    param_names: &[&str],
    body_expression: Expression<'a>,
) -> Expression<'a> {
    let params = ast.alloc_formal_parameters(
        SPAN,
        FormalParameterKind::FormalParameter,
        {
            let mut items = ast.vec_with_capacity(param_names.len());
            for param_name in param_names {
                items.push(ast.formal_parameter(
                    SPAN,
                    ast.vec(),
                    ast.binding_pattern_binding_identifier(SPAN, ast.ident(*param_name)),
                    NONE,
                    NONE,
                    false,
                    None,
                    false,
                    false,
                ));
            }
            items
        },
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
