mod consts;
mod exports;
mod script;

use std::collections::BTreeSet;

use lux_ast::analysis::{AnalysisTables, ScriptTarget};
use lux_ast::template::root::Root;
use oxc_allocator::Allocator;
use oxc_allocator::CloneIn;
use oxc_ast::AstBuilder;
use oxc_ast::ast::{ImportDeclaration, ImportDeclarationSpecifier, Statement};
use oxc_codegen::Codegen;
use oxc_span::{SPAN, SourceType};

use self::consts::{
    LUX_CSS, LUX_CSS_HASH, LUX_CSS_SCOPE, LUX_HAS_DYNAMIC, LUX_TEMPLATE, optional_string_expr,
    push_const, push_runtime_helpers,
};
use self::exports::{default_export_statement, named_export_statement};
use self::script::{
    collect_instance_runtime_binding_names, collect_instance_runtime_statements,
    collect_module_runtime_statements,
};
use super::template::{RuntimeScope, build_render_expression, render_fragment_template};

pub(super) fn render(
    root: &Root<'_>,
    analysis: &AnalysisTables,
    css: Option<&str>,
    css_hash: Option<&str>,
    css_scope: Option<&str>,
) -> String {
    let template_result = render_fragment_template(&root.fragment);

    let allocator = Allocator::default();
    let ast = AstBuilder::new(&allocator);

    let mut body = ast.vec_with_capacity(16);
    push_import_declarations(ast, &mut body, root, analysis);
    let module_runtime = collect_module_runtime_statements(ast, root);
    body.extend(module_runtime);

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
    if template_result.has_dynamic {
        push_runtime_helpers(ast, &mut body);
    }

    body.push(named_export_statement(ast));
    let instance_runtime = collect_instance_runtime_statements(ast, root);
    let mut scope_names = collect_instance_import_names(analysis);
    scope_names.extend(collect_instance_runtime_binding_names(&instance_runtime));
    let scope = RuntimeScope::from_names(scope_names);
    let render_expression = if template_result.has_dynamic {
        build_render_expression(ast, &root.fragment, &scope)
    } else {
        ast.expression_identifier(SPAN, ast.ident(LUX_TEMPLATE))
    };
    body.push(default_export_statement(
        ast,
        render_expression,
        instance_runtime,
    ));

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

fn push_import_declarations<'a>(
    ast: AstBuilder<'a>,
    body: &mut oxc_allocator::Vec<'a, Statement<'a>>,
    root: &Root<'_>,
    analysis: &AnalysisTables,
) {
    let module_import_spans = collect_import_spans(analysis, ScriptTarget::Module);
    let instance_import_spans = collect_import_spans(analysis, ScriptTarget::Instance);

    if let Some(instance_script) = &root.instance {
        push_script_import_declarations(
            ast,
            body,
            &instance_script.content.body,
            &instance_import_spans,
        );
    }
    if let Some(module_script) = &root.module {
        push_script_import_declarations(
            ast,
            body,
            &module_script.content.body,
            &module_import_spans,
        );
    }
}

fn collect_import_spans(analysis: &AnalysisTables, target: ScriptTarget) -> BTreeSet<(u32, u32)> {
    analysis
        .script_imports
        .iter()
        .filter(|item| item.target == target)
        .map(|item| (item.span.start, item.span.end))
        .collect()
}

fn push_script_import_declarations<'a>(
    ast: AstBuilder<'a>,
    body: &mut oxc_allocator::Vec<'a, Statement<'a>>,
    statements: &[Statement<'_>],
    import_spans: &BTreeSet<(u32, u32)>,
) {
    for statement in statements {
        let Statement::ImportDeclaration(declaration) = statement else {
            continue;
        };

        if import_spans.contains(&(declaration.span.start, declaration.span.end)) {
            if let Some(import_statement) = sanitize_import_statement(ast, declaration) {
                body.push(import_statement);
            }
        }
    }
}

fn sanitize_import_statement<'a>(
    ast: AstBuilder<'a>,
    declaration: &oxc_allocator::Box<'_, ImportDeclaration<'_>>,
) -> Option<Statement<'a>> {
    if declaration.import_kind.is_type() {
        return None;
    }

    let mut cloned = declaration.clone_in(ast.allocator);
    if let Some(specifiers) = &declaration.specifiers {
        if specifiers.is_empty() {
            // Normalize `import {} from "x"` to side-effect form `import "x"` for JS output parity.
            cloned.specifiers = None;
        } else {
            let mut kept = ast.vec_with_capacity(specifiers.len());
            for specifier in specifiers {
                match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                        if specifier.import_kind.is_type() {
                            continue;
                        }
                        kept.push(ImportDeclarationSpecifier::ImportSpecifier(
                            specifier.clone_in(ast.allocator),
                        ));
                    }
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                        kept.push(ImportDeclarationSpecifier::ImportDefaultSpecifier(
                            specifier.clone_in(ast.allocator),
                        ));
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                        kept.push(ImportDeclarationSpecifier::ImportNamespaceSpecifier(
                            specifier.clone_in(ast.allocator),
                        ));
                    }
                }
            }
            if kept.is_empty() {
                return None;
            }
            cloned.specifiers = Some(kept);
        }
    }

    Some(Statement::ImportDeclaration(cloned))
}

fn collect_instance_import_names(analysis: &AnalysisTables) -> Vec<String> {
    let mut names = BTreeSet::new();
    for import in &analysis.script_imports {
        if import.target != ScriptTarget::Instance {
            continue;
        }
        for name in &import.local_names {
            names.insert(name.clone());
        }
    }
    names.into_iter().collect()
}
