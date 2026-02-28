use oxc_allocator::Vec as ArenaVec;
use oxc_ast::{
    AstBuilder, NONE,
    ast::{Expression, Statement, VariableDeclarationKind},
};
use oxc_span::SPAN;

pub(super) const LUX_TEMPLATE: &str = "__lux_template";
pub(super) const LUX_CSS: &str = "__lux_css";
pub(super) const LUX_CSS_HASH: &str = "__lux_css_hash";
pub(super) const LUX_CSS_SCOPE: &str = "__lux_css_scope";
pub(super) const LUX_HAS_DYNAMIC: &str = "__lux_has_dynamic";

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
