mod consts;
mod exports;

use lux_ast::template::root::Root;
use oxc_allocator::Allocator;
use oxc_ast::AstBuilder;
use oxc_codegen::Codegen;
use oxc_span::{SPAN, SourceType};

use self::consts::{
    LUX_CSS, LUX_CSS_HASH, LUX_CSS_SCOPE, LUX_HAS_DYNAMIC, LUX_TEMPLATE, optional_string_expr,
    push_const,
};
use self::exports::{default_export_statement, named_export_statement};
use super::template::{build_render_expression, render_fragment_template};

pub(super) fn render(
    root: &Root<'_>,
    css: Option<&str>,
    css_hash: Option<&str>,
    css_scope: Option<&str>,
) -> String {
    let template_result = render_fragment_template(&root.fragment);

    let allocator = Allocator::default();
    let ast = AstBuilder::new(&allocator);

    let mut body = ast.vec_with_capacity(7);
    push_const(
        ast,
        &mut body,
        LUX_TEMPLATE,
        ast.expression_string_literal(SPAN, ast.atom(template_result.html.as_str()), None),
    );
    push_const(ast, &mut body, LUX_CSS, optional_string_expr(ast, css));
    push_const(
        ast,
        &mut body,
        LUX_CSS_HASH,
        optional_string_expr(ast, css_hash),
    );
    push_const(
        ast,
        &mut body,
        LUX_CSS_SCOPE,
        optional_string_expr(ast, css_scope),
    );
    push_const(
        ast,
        &mut body,
        LUX_HAS_DYNAMIC,
        ast.expression_boolean_literal(SPAN, template_result.has_dynamic),
    );

    body.push(named_export_statement(ast));
    let render_expression = if template_result.has_dynamic {
        build_render_expression(ast, &root.fragment)
    } else {
        ast.expression_identifier(SPAN, ast.ident(LUX_TEMPLATE))
    };
    body.push(default_export_statement(ast, render_expression));

    let program = ast.program(
        SPAN,
        SourceType::mjs(),
        "",
        ast.vec(),
        None,
        ast.vec(),
        body,
    );
    Codegen::new().build(&program).code
}
