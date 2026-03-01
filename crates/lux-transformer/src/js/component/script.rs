use std::collections::BTreeSet;

use lux_ast::template::root::Root;
use oxc_allocator::CloneIn;
use oxc_ast::AstBuilder;
use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, Declaration, ExportNamedDeclaration, Expression,
    Statement,
};
use oxc_span::SPAN;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ScriptTarget {
    Instance,
    Module,
}

pub(super) fn collect_module_runtime_statements<'a>(
    ast: AstBuilder<'a>,
    root: &Root<'_>,
) -> oxc_allocator::Vec<'a, Statement<'a>> {
    let mut statements = ast.vec();
    if root.ts {
        return statements;
    }
    let Some(module_script) = &root.module else {
        return statements;
    };

    for statement in &module_script.content.body {
        if let Some(statement) = sanitize_script_statement(ast, statement, ScriptTarget::Module) {
            statements.push(statement);
        }
    }

    statements
}

pub(super) fn collect_instance_runtime_statements<'a>(
    ast: AstBuilder<'a>,
    root: &Root<'_>,
) -> oxc_allocator::Vec<'a, Statement<'a>> {
    let mut statements = ast.vec();
    if root.ts {
        return statements;
    }
    let Some(instance_script) = &root.instance else {
        return statements;
    };

    for statement in &instance_script.content.body {
        if let Some(statement) = sanitize_script_statement(ast, statement, ScriptTarget::Instance) {
            statements.push(statement);
        }
    }

    statements
}

pub(super) fn collect_instance_runtime_binding_names(statements: &[Statement<'_>]) -> Vec<String> {
    let mut names = BTreeSet::new();
    for statement in statements {
        collect_statement_binding_names(statement, &mut names);
    }
    names.into_iter().collect()
}

fn sanitize_script_statement<'a>(
    ast: AstBuilder<'a>,
    statement: &Statement<'_>,
    target: ScriptTarget,
) -> Option<Statement<'a>> {
    match statement {
        Statement::ImportDeclaration(_) => None,
        Statement::ExportNamedDeclaration(declaration) => {
            sanitize_export_named_statement(ast, declaration, target)
        }
        Statement::ExportDefaultDeclaration(_) | Statement::ExportAllDeclaration(_) => {
            if target == ScriptTarget::Instance {
                None
            } else {
                Some(statement.clone_in(ast.allocator))
            }
        }
        Statement::TSExportAssignment(_) | Statement::TSNamespaceExportDeclaration(_) => None,
        Statement::TSTypeAliasDeclaration(_)
        | Statement::TSInterfaceDeclaration(_)
        | Statement::TSModuleDeclaration(_)
        | Statement::TSImportEqualsDeclaration(_)
        | Statement::TSGlobalDeclaration(_)
        | Statement::TSEnumDeclaration(_) => None,
        Statement::VariableDeclaration(declaration) => {
            sanitize_variable_declaration_statement(ast, declaration)
        }
        Statement::ExpressionStatement(expression_statement) => {
            if is_discardable_rune_expression(&expression_statement.expression) {
                None
            } else {
                Some(statement.clone_in(ast.allocator))
            }
        }
        _ => Some(statement.clone_in(ast.allocator)),
    }
}

fn sanitize_export_named_statement<'a>(
    ast: AstBuilder<'a>,
    declaration: &oxc_allocator::Box<'_, ExportNamedDeclaration<'_>>,
    target: ScriptTarget,
) -> Option<Statement<'a>> {
    if declaration.export_kind.is_type() {
        return None;
    }

    if target == ScriptTarget::Instance {
        let declaration = declaration.declaration.as_ref()?;
        return sanitize_declaration_to_statement(ast, declaration);
    }

    let mut cloned = declaration.clone_in(ast.allocator);
    if let Some(inner) = &cloned.declaration {
        let inner_statement = sanitize_declaration_to_statement(ast, inner)?;
        let extracted = statement_to_declaration(inner_statement)?;
        cloned.declaration = Some(extracted);
    }

    if declaration
        .specifiers
        .iter()
        .any(|specifier| !specifier.export_kind.is_value())
    {
        let mut filtered = ast.vec_with_capacity(declaration.specifiers.len());
        for specifier in &declaration.specifiers {
            if specifier.export_kind.is_value() {
                filtered.push(specifier.clone_in(ast.allocator));
            }
        }
        cloned.specifiers = filtered;
    }

    if cloned.declaration.is_none() && cloned.specifiers.is_empty() {
        return None;
    }

    Some(Statement::ExportNamedDeclaration(cloned))
}

fn sanitize_declaration_to_statement<'a>(
    ast: AstBuilder<'a>,
    declaration: &Declaration<'_>,
) -> Option<Statement<'a>> {
    match declaration {
        Declaration::VariableDeclaration(declaration) => {
            sanitize_variable_declaration_statement(ast, declaration)
        }
        Declaration::FunctionDeclaration(function) => Some(Statement::FunctionDeclaration(
            function.clone_in(ast.allocator),
        )),
        Declaration::ClassDeclaration(class) => {
            Some(Statement::ClassDeclaration(class.clone_in(ast.allocator)))
        }
        Declaration::TSTypeAliasDeclaration(_)
        | Declaration::TSInterfaceDeclaration(_)
        | Declaration::TSModuleDeclaration(_)
        | Declaration::TSImportEqualsDeclaration(_)
        | Declaration::TSGlobalDeclaration(_)
        | Declaration::TSEnumDeclaration(_) => None,
    }
}

fn statement_to_declaration(statement: Statement<'_>) -> Option<Declaration<'_>> {
    match statement {
        Statement::VariableDeclaration(declaration) => {
            Some(Declaration::VariableDeclaration(declaration))
        }
        Statement::FunctionDeclaration(function) => {
            Some(Declaration::FunctionDeclaration(function))
        }
        Statement::ClassDeclaration(class) => Some(Declaration::ClassDeclaration(class)),
        _ => None,
    }
}

fn sanitize_variable_declaration_statement<'a>(
    ast: AstBuilder<'a>,
    declaration: &oxc_allocator::Box<'_, oxc_ast::ast::VariableDeclaration<'_>>,
) -> Option<Statement<'a>> {
    if declaration.declare {
        return None;
    }

    let mut cloned = declaration.clone_in(ast.allocator);
    for declarator in &mut cloned.declarations {
        if let Some(init) = declarator.init.take() {
            declarator.init = Some(rewrite_rune_initializer(ast, init));
        }
    }
    Some(Statement::VariableDeclaration(cloned))
}

fn rewrite_rune_initializer<'a>(ast: AstBuilder<'a>, init: Expression<'a>) -> Expression<'a> {
    let Expression::CallExpression(call) = &init else {
        return init;
    };

    let Some(name) = extract_rune_name(&call.callee) else {
        return init;
    };

    match name.as_str() {
        "$state" | "$state.raw" | "$derived" => first_call_argument_expression(ast, call)
            .unwrap_or_else(|| ast.expression_identifier(SPAN, ast.ident("undefined"))),
        "$derived.by" => {
            let Some(argument) = first_call_argument_expression(ast, call) else {
                return ast.expression_identifier(SPAN, ast.ident("undefined"));
            };
            ast.expression_call(SPAN, argument, oxc_ast::NONE, ast.vec(), false)
        }
        "$props" => ast.expression_identifier(SPAN, ast.ident("_props")),
        _ => init,
    }
}

fn first_call_argument_expression<'a>(
    ast: AstBuilder<'a>,
    call: &CallExpression<'a>,
) -> Option<Expression<'a>> {
    call.arguments.iter().find_map(|argument| {
        if matches!(argument, Argument::SpreadElement(_)) {
            return None;
        }
        argument
            .as_expression()
            .map(|expression| expression.clone_in(ast.allocator))
    })
}

fn is_discardable_rune_expression(expression: &Expression<'_>) -> bool {
    let Expression::CallExpression(call) = expression else {
        return false;
    };

    matches!(
        extract_rune_name(&call.callee).as_deref(),
        Some(
            "$effect"
                | "$effect.pre"
                | "$effect.tracking"
                | "$effect.root"
                | "$effect.pending"
                | "$inspect"
                | "$inspect.trace"
        )
    )
}

fn extract_rune_name(callee: &Expression<'_>) -> Option<String> {
    match callee {
        Expression::Identifier(identifier) => Some(identifier.name.as_str().to_owned()),
        Expression::StaticMemberExpression(member) => {
            let object_name = extract_rune_name(&member.object)?;
            Some(format!("{object_name}.{}", member.property.name.as_str()))
        }
        Expression::ParenthesizedExpression(expression) => {
            extract_rune_name(&expression.expression)
        }
        _ => None,
    }
}

fn collect_statement_binding_names(statement: &Statement<'_>, names: &mut BTreeSet<String>) {
    match statement {
        Statement::VariableDeclaration(declaration) => {
            for declarator in &declaration.declarations {
                collect_binding_pattern_names(&declarator.id, names);
            }
        }
        Statement::FunctionDeclaration(function) => {
            if let Some(id) = &function.id {
                names.insert(id.name.as_str().to_owned());
            }
        }
        Statement::ClassDeclaration(class) => {
            if let Some(id) = &class.id {
                names.insert(id.name.as_str().to_owned());
            }
        }
        _ => {}
    }
}

fn collect_binding_pattern_names(pattern: &BindingPattern<'_>, names: &mut BTreeSet<String>) {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => {
            names.insert(identifier.name.as_str().to_owned());
        }
        BindingPattern::ObjectPattern(pattern) => {
            for property in &pattern.properties {
                collect_binding_pattern_names(&property.value, names);
            }
            if let Some(rest) = &pattern.rest {
                collect_binding_pattern_names(&rest.argument, names);
            }
        }
        BindingPattern::ArrayPattern(pattern) => {
            for element in &pattern.elements {
                if let Some(element) = element {
                    collect_binding_pattern_names(element, names);
                }
            }
            if let Some(rest) = &pattern.rest {
                collect_binding_pattern_names(&rest.argument, names);
            }
        }
        BindingPattern::AssignmentPattern(pattern) => {
            collect_binding_pattern_names(&pattern.left, names);
        }
    }
}
