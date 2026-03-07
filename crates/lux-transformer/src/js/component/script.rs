use std::collections::BTreeSet;

use lux_ast::template::root::Root;
use oxc_allocator::CloneIn;
use oxc_ast::ast::{
    AccessorProperty, Argument, ArrowFunctionExpression, AssignmentTarget,
    AssignmentTargetMaybeDefault, AssignmentTargetProperty, BindingPattern, CallExpression,
    CatchParameter, Class, Declaration, ExportNamedDeclaration, Expression, FormalParameter,
    Function, MethodDefinition, PropertyDefinition, Statement, VariableDeclarator,
};
use oxc_ast::{AstBuilder, NONE};
use oxc_ast_visit::{VisitMut, walk_mut};
use oxc_span::SPAN;
use oxc_syntax::scope::ScopeFlags;

use crate::js::component::LUX_REST_PROPS;

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
    let Some(module_script) = &root.module else {
        return statements;
    };

    for statement in &module_script.content.body {
        if let Some(mut statement) = sanitize_script_statement(ast, statement, ScriptTarget::Module)
        {
            strip_typescript_from_statement(ast, &mut statement);
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
    let mut declared_names = BTreeSet::new();
    let mut reactive_names = BTreeSet::new();
    let Some(instance_script) = &root.instance else {
        return statements;
    };
    let legacy_exported_props = collect_instance_exported_prop_names(root);

    for statement in &instance_script.content.body {
        if let Some((names, mut statement)) = sanitize_reactive_statement(ast, statement) {
            strip_typescript_from_statement(ast, &mut statement);
            reactive_names.extend(names);
            collect_statement_binding_names(&statement, &mut declared_names);
            statements.push(statement);
            continue;
        }

        if let Some(mut statement) =
            sanitize_script_statement(ast, statement, ScriptTarget::Instance)
        {
            strip_typescript_from_statement(ast, &mut statement);
            collect_statement_binding_names(&statement, &mut declared_names);
            statements.push(statement);
        }
    }

    if reactive_names.is_empty() {
        return prepend_legacy_helper_declarations(ast, statements, &legacy_exported_props);
    }

    let missing_reactive_names = reactive_names
        .into_iter()
        .filter(|name| !declared_names.contains(name))
        .collect::<Vec<_>>();
    if missing_reactive_names.is_empty() {
        return prepend_legacy_helper_declarations(ast, statements, &legacy_exported_props);
    }

    let mut prefixed = ast.vec_with_capacity(statements.len() + missing_reactive_names.len());
    for name in missing_reactive_names {
        prefixed.push(build_let_declaration_statement(ast, &name));
    }
    prefixed.extend(statements);
    prepend_legacy_helper_declarations(ast, prefixed, &legacy_exported_props)
}

pub(super) fn collect_runtime_binding_names(statements: &[Statement<'_>]) -> Vec<String> {
    let mut names = BTreeSet::new();
    for statement in statements {
        collect_statement_binding_names(statement, &mut names);
    }
    names.into_iter().collect()
}

pub(super) fn needs_rest_props_runtime(root: &Root<'_>) -> bool {
    !collect_instance_exported_prop_names(root).is_empty()
}

pub(super) fn rewrite_server_store_subscriptions<'a>(
    ast: AstBuilder<'a>,
    statements: &mut oxc_allocator::Vec<'a, Statement<'a>>,
) {
    let mut rewriter = ServerStoreSubscriptionRewriter { ast };
    for statement in statements {
        rewriter.visit_statement(statement);
    }
}

fn sanitize_reactive_statement<'a>(
    ast: AstBuilder<'a>,
    statement: &Statement<'_>,
) -> Option<(Vec<String>, Statement<'a>)> {
    let Statement::LabeledStatement(labeled) = statement else {
        return None;
    };
    if labeled.label.name.as_str() != "$" {
        return None;
    }

    let mut names = BTreeSet::new();
    collect_reactive_binding_names(&labeled.body, &mut names);
    Some((
        names.into_iter().collect(),
        labeled.body.clone_in(ast.allocator),
    ))
}

fn collect_instance_exported_prop_names(root: &Root<'_>) -> Vec<String> {
    let mut names = BTreeSet::new();
    let Some(instance_script) = &root.instance else {
        return names.into_iter().collect();
    };

    for statement in &instance_script.content.body {
        let Statement::ExportNamedDeclaration(declaration) = statement else {
            continue;
        };
        let Some(Declaration::VariableDeclaration(declaration)) = &declaration.declaration else {
            continue;
        };
        for declarator in &declaration.declarations {
            collect_binding_pattern_names(&declarator.id, &mut names);
        }
    }

    names.into_iter().collect()
}

fn prepend_legacy_helper_declarations<'a>(
    ast: AstBuilder<'a>,
    statements: oxc_allocator::Vec<'a, Statement<'a>>,
    exported_props: &[String],
) -> oxc_allocator::Vec<'a, Statement<'a>> {
    if exported_props.is_empty() {
        return statements;
    }

    let mut prefixed = ast.vec_with_capacity(statements.len() + 2);
    prefixed.push(build_props_alias_statement(ast));
    prefixed.push(build_rest_props_statement(ast, exported_props));
    prefixed.extend(statements);
    prefixed
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
        return match declaration {
            Declaration::VariableDeclaration(declaration) => {
                sanitize_instance_exported_variable_declaration_statement(ast, declaration)
            }
            _ => sanitize_declaration_to_statement(ast, declaration),
        };
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

fn sanitize_instance_exported_variable_declaration_statement<'a>(
    ast: AstBuilder<'a>,
    declaration: &oxc_allocator::Box<'_, oxc_ast::ast::VariableDeclaration<'_>>,
) -> Option<Statement<'a>> {
    if declaration.declare {
        return None;
    }

    let mut cloned = declaration.clone_in(ast.allocator);
    for declarator in &mut cloned.declarations {
        declarator.init =
            rewrite_exported_prop_initializer(ast, &declarator.id, declarator.init.take());
    }
    Some(Statement::VariableDeclaration(cloned))
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

fn rewrite_exported_prop_initializer<'a>(
    ast: AstBuilder<'a>,
    pattern: &BindingPattern<'a>,
    init: Option<Expression<'a>>,
) -> Option<Expression<'a>> {
    let BindingPattern::BindingIdentifier(identifier) = pattern else {
        return init.map(|expression| rewrite_rune_initializer(ast, expression));
    };

    let prop_expression = ast.member_expression_static(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident("_props")),
        ast.identifier_name(SPAN, ast.ident(identifier.name.as_str())),
        false,
    );
    let prop_fallback_test = prop_expression.clone_in(ast.allocator);
    let prop_value: Expression<'a> = prop_expression.clone_in(ast.allocator).into();

    Some(match init {
        Some(init) => ast.expression_conditional(
            SPAN,
            ast.expression_binary(
                SPAN,
                prop_fallback_test.into(),
                oxc_ast::ast::BinaryOperator::StrictEquality,
                ast.expression_identifier(SPAN, ast.ident("undefined")),
            ),
            rewrite_rune_initializer(ast, init),
            prop_value,
        ),
        None => prop_value,
    })
}

fn rewrite_rune_initializer<'a>(ast: AstBuilder<'a>, init: Expression<'a>) -> Expression<'a> {
    if let Some(inner) = strip_typescript_expression_wrapper(ast, &init) {
        return rewrite_rune_initializer(ast, inner);
    }

    rewrite_rune_call_expression(ast, &init).unwrap_or(init)
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

fn rewrite_rune_call_expression<'a>(
    ast: AstBuilder<'a>,
    expression: &Expression<'a>,
) -> Option<Expression<'a>> {
    let Expression::CallExpression(call) = expression else {
        return None;
    };

    let name = extract_rune_name(&call.callee)?;
    match name.as_str() {
        "$state" | "$state.raw" | "$derived" | "$state.snapshot" => Some(
            first_call_argument_expression(ast, call)
                .unwrap_or_else(|| ast.expression_identifier(SPAN, ast.ident("undefined"))),
        ),
        "$derived.by" => {
            let argument = first_call_argument_expression(ast, call)
                .unwrap_or_else(|| ast.expression_identifier(SPAN, ast.ident("undefined")));
            Some(ast.expression_call(SPAN, argument, oxc_ast::NONE, ast.vec(), false))
        }
        "$props" => Some(ast.expression_identifier(SPAN, ast.ident("_props"))),
        "$props.id" => Some(ast.expression_call(
            SPAN,
            ast.expression_identifier(SPAN, ast.ident("__lux_props_id")),
            oxc_ast::NONE,
            ast.vec(),
            false,
        )),
        "$bindable" | "$effect" | "$effect.pre" | "$effect.tracking" | "$effect.root"
        | "$effect.pending" | "$inspect" | "$inspect.trace" | "$host" => {
            Some(ast.expression_identifier(SPAN, ast.ident("undefined")))
        }
        _ => None,
    }
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

fn strip_typescript_expression_wrapper<'a>(
    ast: AstBuilder<'a>,
    expression: &Expression<'a>,
) -> Option<Expression<'a>> {
    match expression {
        Expression::TSAsExpression(wrapper) => Some(wrapper.expression.clone_in(ast.allocator)),
        Expression::TSSatisfiesExpression(wrapper) => {
            Some(wrapper.expression.clone_in(ast.allocator))
        }
        Expression::TSTypeAssertion(wrapper) => Some(wrapper.expression.clone_in(ast.allocator)),
        Expression::TSNonNullExpression(wrapper) => {
            Some(wrapper.expression.clone_in(ast.allocator))
        }
        Expression::TSInstantiationExpression(wrapper) => {
            Some(wrapper.expression.clone_in(ast.allocator))
        }
        _ => None,
    }
}

fn strip_typescript_from_statement<'a>(ast: AstBuilder<'a>, statement: &mut Statement<'a>) {
    let mut eraser = TypeScriptEraser { ast };
    eraser.visit_statement(statement);
}

struct TypeScriptEraser<'a> {
    ast: AstBuilder<'a>,
}

struct ServerStoreSubscriptionRewriter<'a> {
    ast: AstBuilder<'a>,
}

impl<'a> VisitMut<'a> for TypeScriptEraser<'a> {
    fn visit_expression(&mut self, expression: &mut Expression<'a>) {
        loop {
            let mut changed = false;
            while let Some(inner) = strip_typescript_expression_wrapper(self.ast, expression) {
                *expression = inner;
                changed = true;
            }
            if let Some(inner) = rewrite_rune_call_expression(self.ast, expression) {
                *expression = inner;
                changed = true;
            }
            if !changed {
                break;
            }
        }
        walk_mut::walk_expression(self, expression);
    }

    fn visit_call_expression(&mut self, expression: &mut CallExpression<'a>) {
        expression.type_arguments = None;
        walk_mut::walk_call_expression(self, expression);
    }

    fn visit_new_expression(&mut self, expression: &mut oxc_ast::ast::NewExpression<'a>) {
        expression.type_arguments = None;
        walk_mut::walk_new_expression(self, expression);
    }

    fn visit_variable_declarator(&mut self, declarator: &mut VariableDeclarator<'a>) {
        declarator.type_annotation = None;
        declarator.definite = false;
        walk_mut::walk_variable_declarator(self, declarator);
    }

    fn visit_catch_parameter(&mut self, parameter: &mut CatchParameter<'a>) {
        parameter.type_annotation = None;
        walk_mut::walk_catch_parameter(self, parameter);
    }

    fn visit_function(&mut self, function: &mut Function<'a>, flags: ScopeFlags) {
        function.declare = false;
        function.type_parameters = None;
        function.this_param = None;
        function.return_type = None;
        walk_mut::walk_function(self, function, flags);
    }

    fn visit_formal_parameter(&mut self, parameter: &mut FormalParameter<'a>) {
        parameter.decorators = self.ast.vec();
        parameter.type_annotation = None;
        parameter.optional = false;
        parameter.accessibility = None;
        walk_mut::walk_formal_parameter(self, parameter);
    }

    fn visit_arrow_function_expression(&mut self, expression: &mut ArrowFunctionExpression<'a>) {
        expression.type_parameters = None;
        expression.return_type = None;
        walk_mut::walk_arrow_function_expression(self, expression);
    }

    fn visit_class(&mut self, class: &mut Class<'a>) {
        class.type_parameters = None;
        class.super_type_arguments = None;
        class.implements = self.ast.vec();
        walk_mut::walk_class(self, class);
    }

    fn visit_method_definition(&mut self, definition: &mut MethodDefinition<'a>) {
        definition.r#override = false;
        definition.optional = false;
        definition.accessibility = None;
        walk_mut::walk_method_definition(self, definition);
    }

    fn visit_property_definition(&mut self, definition: &mut PropertyDefinition<'a>) {
        definition.type_annotation = None;
        definition.declare = false;
        definition.r#override = false;
        definition.optional = false;
        definition.definite = false;
        definition.readonly = false;
        definition.accessibility = None;
        walk_mut::walk_property_definition(self, definition);
    }

    fn visit_accessor_property(&mut self, property: &mut AccessorProperty<'a>) {
        property.type_annotation = None;
        property.r#override = false;
        property.definite = false;
        property.accessibility = None;
        walk_mut::walk_accessor_property(self, property);
    }
}

impl<'a> VisitMut<'a> for ServerStoreSubscriptionRewriter<'a> {
    fn visit_expression(&mut self, expression: &mut Expression<'a>) {
        walk_mut::walk_expression(self, expression);

        let Expression::Identifier(identifier) = expression else {
            return;
        };
        let name = identifier.name.as_str();
        if !name.starts_with('$') || name.starts_with("$$") || name.len() < 2 {
            return;
        }

        let store_name = &name[1..];
        *expression = self.ast.expression_call(
            SPAN,
            self.ast
                .expression_identifier(SPAN, self.ast.ident("__lux_store_get")),
            NONE,
            self.ast.vec_from_array([
                self.ast
                    .expression_identifier(SPAN, self.ast.ident("__lux_store_values"))
                    .into(),
                self.ast
                    .expression_string_literal(SPAN, self.ast.atom(name), None)
                    .into(),
                self.ast
                    .expression_identifier(SPAN, self.ast.ident(store_name))
                    .into(),
            ]),
            false,
        );
    }
}

fn collect_statement_binding_names(statement: &Statement<'_>, names: &mut BTreeSet<String>) {
    match statement {
        Statement::ExportNamedDeclaration(declaration) => {
            if let Some(inner) = &declaration.declaration {
                collect_declaration_binding_names(inner, names);
            }
        }
        Statement::VariableDeclaration(declaration) => {
            collect_variable_declaration_binding_names(declaration, names);
        }
        Statement::FunctionDeclaration(function) => collect_function_binding_name(function, names),
        Statement::ClassDeclaration(class) => collect_class_binding_name(class, names),
        _ => {}
    }
}

fn collect_declaration_binding_names(declaration: &Declaration<'_>, names: &mut BTreeSet<String>) {
    match declaration {
        Declaration::VariableDeclaration(declaration) => {
            collect_variable_declaration_binding_names(declaration, names);
        }
        Declaration::FunctionDeclaration(function) => {
            collect_function_binding_name(function, names)
        }
        Declaration::ClassDeclaration(class) => collect_class_binding_name(class, names),
        Declaration::TSTypeAliasDeclaration(_)
        | Declaration::TSInterfaceDeclaration(_)
        | Declaration::TSModuleDeclaration(_)
        | Declaration::TSImportEqualsDeclaration(_)
        | Declaration::TSGlobalDeclaration(_)
        | Declaration::TSEnumDeclaration(_) => {}
    }
}

fn collect_variable_declaration_binding_names(
    declaration: &oxc_allocator::Box<'_, oxc_ast::ast::VariableDeclaration<'_>>,
    names: &mut BTreeSet<String>,
) {
    for declarator in &declaration.declarations {
        collect_binding_pattern_names(&declarator.id, names);
    }
}

fn collect_function_binding_name(
    function: &oxc_allocator::Box<'_, Function<'_>>,
    names: &mut BTreeSet<String>,
) {
    if let Some(id) = &function.id {
        names.insert(id.name.as_str().to_owned());
    }
}

fn collect_class_binding_name(
    class: &oxc_allocator::Box<'_, Class<'_>>,
    names: &mut BTreeSet<String>,
) {
    if let Some(id) = &class.id {
        names.insert(id.name.as_str().to_owned());
    }
}

fn collect_reactive_binding_names(statement: &Statement<'_>, names: &mut BTreeSet<String>) {
    match statement {
        Statement::ExpressionStatement(expression_statement) => {
            let Expression::AssignmentExpression(assignment) = &expression_statement.expression
            else {
                return;
            };
            collect_assignment_target_names(&assignment.left, names);
        }
        Statement::BlockStatement(block) => {
            for statement in &block.body {
                collect_reactive_binding_names(statement, names);
            }
        }
        _ => {}
    }
}

fn collect_assignment_target_names(target: &AssignmentTarget<'_>, names: &mut BTreeSet<String>) {
    match target {
        AssignmentTarget::AssignmentTargetIdentifier(identifier) => {
            names.insert(identifier.name.as_str().to_owned());
        }
        AssignmentTarget::ArrayAssignmentTarget(pattern) => {
            for element in &pattern.elements {
                if let Some(element) = element {
                    collect_assignment_target_maybe_default_names(element, names);
                }
            }
            if let Some(rest) = &pattern.rest {
                collect_assignment_target_names(&rest.target, names);
            }
        }
        AssignmentTarget::ObjectAssignmentTarget(pattern) => {
            for property in &pattern.properties {
                match property {
                    AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(property) => {
                        names.insert(property.binding.name.as_str().to_owned());
                    }
                    AssignmentTargetProperty::AssignmentTargetPropertyProperty(property) => {
                        collect_assignment_target_maybe_default_names(&property.binding, names);
                    }
                }
            }
            if let Some(rest) = &pattern.rest {
                collect_assignment_target_names(&rest.target, names);
            }
        }
        AssignmentTarget::TSAsExpression(_)
        | AssignmentTarget::TSSatisfiesExpression(_)
        | AssignmentTarget::TSNonNullExpression(_)
        | AssignmentTarget::TSTypeAssertion(_)
        | AssignmentTarget::ComputedMemberExpression(_)
        | AssignmentTarget::StaticMemberExpression(_)
        | AssignmentTarget::PrivateFieldExpression(_) => {}
    }
}

fn collect_assignment_target_maybe_default_names(
    target: &AssignmentTargetMaybeDefault<'_>,
    names: &mut BTreeSet<String>,
) {
    match target {
        AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(target) => {
            collect_assignment_target_names(&target.binding, names);
        }
        AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(identifier) => {
            names.insert(identifier.name.as_str().to_owned());
        }
        AssignmentTargetMaybeDefault::ArrayAssignmentTarget(pattern) => {
            for element in &pattern.elements {
                if let Some(element) = element {
                    collect_assignment_target_maybe_default_names(element, names);
                }
            }
            if let Some(rest) = &pattern.rest {
                collect_assignment_target_names(&rest.target, names);
            }
        }
        AssignmentTargetMaybeDefault::ObjectAssignmentTarget(pattern) => {
            for property in &pattern.properties {
                match property {
                    AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(property) => {
                        names.insert(property.binding.name.as_str().to_owned());
                    }
                    AssignmentTargetProperty::AssignmentTargetPropertyProperty(property) => {
                        collect_assignment_target_maybe_default_names(&property.binding, names);
                    }
                }
            }
            if let Some(rest) = &pattern.rest {
                collect_assignment_target_names(&rest.target, names);
            }
        }
        AssignmentTargetMaybeDefault::TSAsExpression(_)
        | AssignmentTargetMaybeDefault::TSSatisfiesExpression(_)
        | AssignmentTargetMaybeDefault::TSNonNullExpression(_)
        | AssignmentTargetMaybeDefault::TSTypeAssertion(_)
        | AssignmentTargetMaybeDefault::ComputedMemberExpression(_)
        | AssignmentTargetMaybeDefault::StaticMemberExpression(_)
        | AssignmentTargetMaybeDefault::PrivateFieldExpression(_) => {}
    }
}

fn build_let_declaration_statement<'a>(ast: AstBuilder<'a>, name: &str) -> Statement<'a> {
    ast.declaration_variable(
        SPAN,
        oxc_ast::ast::VariableDeclarationKind::Let,
        ast.vec1(ast.variable_declarator(
            SPAN,
            oxc_ast::ast::VariableDeclarationKind::Let,
            ast.binding_pattern_binding_identifier(SPAN, ast.ident(name)),
            NONE,
            None,
            false,
        )),
        false,
    )
    .into()
}

fn build_props_alias_statement<'a>(ast: AstBuilder<'a>) -> Statement<'a> {
    ast.declaration_variable(
        SPAN,
        oxc_ast::ast::VariableDeclarationKind::Const,
        ast.vec1(ast.variable_declarator(
            SPAN,
            oxc_ast::ast::VariableDeclarationKind::Const,
            ast.binding_pattern_binding_identifier(SPAN, ast.ident("$$props")),
            NONE,
            Some(ast.expression_identifier(SPAN, ast.ident("_props"))),
            false,
        )),
        false,
    )
    .into()
}

fn build_rest_props_statement<'a>(ast: AstBuilder<'a>, exported_props: &[String]) -> Statement<'a> {
    let mut exclude_items = ast.vec_with_capacity(exported_props.len());
    for name in exported_props {
        exclude_items.push(
            ast.expression_string_literal(SPAN, ast.atom(name.as_str()), None)
                .into(),
        );
    }

    ast.declaration_variable(
        SPAN,
        oxc_ast::ast::VariableDeclarationKind::Const,
        ast.vec1(ast.variable_declarator(
            SPAN,
            oxc_ast::ast::VariableDeclarationKind::Const,
            ast.binding_pattern_binding_identifier(SPAN, ast.ident("$$restProps")),
            NONE,
            Some(ast.expression_call(
                SPAN,
                ast.expression_identifier(SPAN, ast.ident(LUX_REST_PROPS)),
                NONE,
                ast.vec_from_array([
                    ast.expression_identifier(SPAN, ast.ident("_props")).into(),
                    ast.expression_array(SPAN, exclude_items).into(),
                ]),
                false,
            )),
            false,
        )),
        false,
    )
    .into()
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
