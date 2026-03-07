use std::collections::HashMap;

use rustc_hash::FxHashSet;

use lux_ast::analysis::{
    AnalysisDiagnostic, AnalysisDiagnosticCode, AnalysisSeverity, AnalysisTables,
    ScriptImportAnalysis, ScriptReferenceAnalysis, ScriptRuneAnalysis, ScriptRuneKind,
    ScriptScopeAnalysis, ScriptSymbolAnalysis, ScriptTarget,
};
use lux_utils::runes::{is_rune, is_state_creation_rune};
use oxc_ast::AstKind;
use oxc_ast::ast::{
    AssignmentTarget, AssignmentTargetMaybeDefault, AssignmentTargetProperty, BindingPattern,
    CallExpression, Class, ClassElement, Declaration, ExportDefaultDeclarationKind, Expression,
    ImportDeclarationSpecifier, MethodDefinition, MethodDefinitionKind, Program,
    SimpleAssignmentTarget, Statement, UpdateExpression, VariableDeclaration,
};
use oxc_ast_visit::{Visit, walk};
use oxc_semantic::{ReferenceId, Semantic, SemanticBuilder};
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::AssignmentOperator;

pub(super) fn analyze_program(
    program: &Program<'_>,
    target: ScriptTarget,
    is_custom_element: bool,
    tables: &mut AnalysisTables,
) {
    let semantic_result = SemanticBuilder::new().build(program);
    let semantic = semantic_result.semantic;
    let runes = collect_runes(program);

    add_scope_records(&semantic, target, tables);
    add_symbol_records(&semantic, target, tables);
    add_reference_records(&semantic, target, tables);
    add_rune_records(&runes, target, tables);
    add_rune_argument_diagnostics(&runes, tables);
    add_rune_placement_diagnostics(program, target, is_custom_element, &runes, tables);
    add_module_export_rune_diagnostics(program, target, tables);
    add_import_records(program, target, tables);
}

fn add_scope_records(semantic: &Semantic<'_>, target: ScriptTarget, tables: &mut AnalysisTables) {
    let scoping = semantic.scoping();

    for scope_id in scoping.scope_descendants_from_root() {
        tables.script_scopes.push(ScriptScopeAnalysis {
            target,
            id: scope_id.index() as u32,
            parent: scoping
                .scope_parent_id(scope_id)
                .map(|parent| parent.index() as u32),
            flags: scoping.scope_flags(scope_id).bits(),
            node_id: scoping.get_node_id(scope_id).index() as u32,
        });
    }
}

fn add_symbol_records(semantic: &Semantic<'_>, target: ScriptTarget, tables: &mut AnalysisTables) {
    let scoping = semantic.scoping();

    for symbol_id in scoping.symbol_ids() {
        tables.script_symbols.push(ScriptSymbolAnalysis {
            target,
            id: symbol_id.index() as u32,
            name: scoping.symbol_name(symbol_id).to_owned(),
            scope_id: scoping.symbol_scope_id(symbol_id).index() as u32,
            declaration_node_id: scoping.symbol_declaration(symbol_id).index() as u32,
            declaration_span: scoping.symbol_span(symbol_id),
            flags: scoping.symbol_flags(symbol_id).bits(),
            mutated: scoping.symbol_is_mutated(symbol_id),
            unused: scoping.symbol_is_unused(symbol_id),
        });
    }
}

fn add_reference_records(
    semantic: &Semantic<'_>,
    target: ScriptTarget,
    tables: &mut AnalysisTables,
) {
    let scoping = semantic.scoping();
    let mut seen_reference_ids: FxHashSet<usize> = FxHashSet::default();

    for symbol_id in scoping.symbol_ids() {
        for &reference_id in scoping.get_resolved_reference_ids(symbol_id) {
            if seen_reference_ids.insert(reference_id.index()) {
                add_reference_record(semantic, target, reference_id, tables);
            }
        }
    }

    for unresolved_group in scoping.root_unresolved_references_ids() {
        for reference_id in unresolved_group {
            if seen_reference_ids.insert(reference_id.index()) {
                add_reference_record(semantic, target, reference_id, tables);
            }
        }
    }
}

fn add_reference_record(
    semantic: &Semantic<'_>,
    target: ScriptTarget,
    reference_id: ReferenceId,
    tables: &mut AnalysisTables,
) {
    let scoping = semantic.scoping();
    let reference = scoping.get_reference(reference_id);
    let node = semantic.nodes().get_node(reference.node_id());

    let AstKind::IdentifierReference(identifier) = node.kind() else {
        return;
    };

    tables.script_references.push(ScriptReferenceAnalysis {
        target,
        id: reference_id.index() as u32,
        name: identifier.name.as_str().to_owned(),
        span: identifier.span,
        scope_id: reference.scope_id().index() as u32,
        symbol_id: reference
            .symbol_id()
            .map(|symbol_id| symbol_id.index() as u32),
        is_read: reference.is_read(),
        is_write: reference.is_write(),
    });
}

fn add_rune_records(runes: &[CollectedRune], target: ScriptTarget, tables: &mut AnalysisTables) {
    for rune in runes {
        let kind = if is_rune(&rune.name) {
            ScriptRuneKind::Known
        } else {
            ScriptRuneKind::Unknown
        };
        tables.script_runes.push(ScriptRuneAnalysis {
            target,
            name: rune.name.clone(),
            kind,
            span: rune.span,
            callee_span: rune.callee_span,
            argument_count: rune.argument_count,
            is_state_creation: is_state_creation_rune(&rune.name),
        });
    }
}

fn add_rune_argument_diagnostics(runes: &[CollectedRune], tables: &mut AnalysisTables) {
    for rune in runes {
        match rune.name.as_str() {
            "$state" | "$state.raw" | "$bindable" => {
                if rune.argument_count > 1 {
                    push_rune_argument_length_diagnostic(
                        tables,
                        &rune.name,
                        "zero or one",
                        rune.callee_span,
                    );
                }
            }
            "$derived" | "$derived.by" | "$effect" | "$effect.pre" | "$effect.root"
            | "$state.snapshot" => {
                if rune.argument_count != 1 {
                    push_rune_argument_length_diagnostic(
                        tables,
                        &rune.name,
                        "exactly one",
                        rune.callee_span,
                    );
                }
            }
            "$inspect" => {
                if rune.argument_count == 0 {
                    push_rune_argument_length_diagnostic(
                        tables,
                        &rune.name,
                        "one or more",
                        rune.callee_span,
                    );
                }
            }
            "$props" | "$props.id" | "$host" | "$effect.tracking" => {
                if rune.argument_count != 0 {
                    tables.diagnostics.push(AnalysisDiagnostic {
                        severity: AnalysisSeverity::Error,
                        code: AnalysisDiagnosticCode::ScriptRuneInvalidArguments,
                        message: format!("`{}` cannot be called with arguments.", rune.name),
                        span: rune.callee_span,
                    });
                }
            }
            _ => {}
        }
    }
}

fn add_rune_placement_diagnostics(
    program: &Program<'_>,
    target: ScriptTarget,
    is_custom_element: bool,
    runes: &[CollectedRune],
    tables: &mut AnalysisTables,
) {
    let allowed_state_spans = collect_allowed_state_rune_spans(program);
    let allowed_props_spans = collect_allowed_props_rune_spans(program);
    let allowed_bindable_spans = collect_allowed_bindable_rune_spans(program, &allowed_props_spans);
    let allowed_effect_spans = collect_allowed_effect_rune_spans(program);

    for rune in runes {
        let allowed = match rune.name.as_str() {
            "$state" | "$state.raw" | "$derived" | "$derived.by" => {
                allowed_state_spans.contains(&(rune.span.start, rune.span.end))
            }
            "$props" | "$props.id" => {
                allowed_props_spans.contains(&(rune.span.start, rune.span.end))
            }
            "$bindable" => allowed_bindable_spans.contains(&(rune.span.start, rune.span.end)),
            "$effect" | "$effect.pre" | "$effect.root" => {
                allowed_effect_spans.contains(&(rune.span.start, rune.span.end))
            }
            "$host" => target == ScriptTarget::Instance && is_custom_element,
            _ => true,
        };
        if allowed {
            continue;
        }

        tables.diagnostics.push(AnalysisDiagnostic {
            severity: AnalysisSeverity::Error,
            code: AnalysisDiagnosticCode::TemplateRuneInvalidPlacement,
            message: rune_invalid_placement_message(&rune.name),
            span: rune.callee_span,
        });
    }
}

fn push_rune_argument_length_diagnostic(
    tables: &mut AnalysisTables,
    rune_name: &str,
    expected: &str,
    span: Span,
) {
    tables.diagnostics.push(AnalysisDiagnostic {
        severity: AnalysisSeverity::Error,
        code: AnalysisDiagnosticCode::ScriptRuneInvalidArgumentsLength,
        message: format!("`{rune_name}` must be called with {expected} argument(s)."),
        span,
    });
}

fn rune_invalid_placement_message(rune_name: &str) -> String {
    match rune_name {
        "$state" | "$state.raw" | "$derived" | "$derived.by" => format!(
            "`{rune_name}(...)` can only be used as a variable declaration initializer, a class field declaration, or the first assignment to a class field at the top level of the constructor."
        ),
        "$props" => {
            "`$props()` can only be used at the top level as a variable declaration initializer."
                .to_string()
        }
        "$props.id" => {
            "`$props.id()` can only be used at the top level as a variable declaration initializer."
                .to_string()
        }
        "$bindable" => {
            "`$bindable()` can only be used inside a top-level `$props()` declaration.".to_string()
        }
        "$effect" | "$effect.pre" | "$effect.root" => {
            format!("`{rune_name}()` can only be used as an expression statement.")
        }
        "$host" => {
            "`$host()` can only be used inside custom element component instances.".to_string()
        }
        _ => format!("`{rune_name}(...)` is used in an invalid placement."),
    }
}

fn add_module_export_rune_diagnostics(
    program: &Program<'_>,
    target: ScriptTarget,
    tables: &mut AnalysisTables,
) {
    if target != ScriptTarget::Module {
        return;
    }

    let rune_bindings = collect_top_level_rune_bindings(program);
    if rune_bindings.is_empty() {
        return;
    }

    let exported_names = collect_exported_names(program);
    if exported_names.is_empty() {
        return;
    }
    let reassigned_identifiers = collect_reassigned_identifiers(program);

    for exported_name in exported_names {
        let Some((rune_name, rune_span)) = rune_bindings.get(&exported_name) else {
            continue;
        };

        match rune_name.as_str() {
            "$derived" | "$derived.by" => {
                tables.diagnostics.push(AnalysisDiagnostic {
                    severity: AnalysisSeverity::Error,
                    code: AnalysisDiagnosticCode::TemplateRuneInvalidPlacement,
                    message: "Cannot export derived state from a module. Export a function returning the derived value instead.".to_string(),
                    span: *rune_span,
                });
            }
            "$state" | "$state.raw" => {
                if reassigned_identifiers.contains(&exported_name) {
                    tables.diagnostics.push(AnalysisDiagnostic {
                        severity: AnalysisSeverity::Error,
                        code: AnalysisDiagnosticCode::TemplateRuneInvalidPlacement,
                        message: "Cannot export reassigned state from a module. Export a function returning the state value instead.".to_string(),
                        span: *rune_span,
                    });
                }
            }
            _ => {}
        }
    }
}

fn collect_top_level_rune_bindings(program: &Program<'_>) -> HashMap<String, (String, Span)> {
    let mut bindings = HashMap::new();

    for statement in &program.body {
        match statement {
            Statement::VariableDeclaration(declaration) => {
                collect_rune_bindings_from_variable_declaration(declaration, &mut bindings);
            }
            Statement::ExportNamedDeclaration(export) => {
                if let Some(Declaration::VariableDeclaration(declaration)) = &export.declaration {
                    collect_rune_bindings_from_variable_declaration(declaration, &mut bindings);
                }
            }
            _ => {}
        }
    }

    bindings
}

fn collect_rune_bindings_from_variable_declaration(
    declaration: &VariableDeclaration<'_>,
    bindings: &mut HashMap<String, (String, Span)>,
) {
    for declarator in &declaration.declarations {
        let Some(identifier) = binding_identifier_name(&declarator.id) else {
            continue;
        };
        let Some(init) = &declarator.init else {
            continue;
        };
        let Some((rune_name, rune_span)) = extract_rune_call(init) else {
            continue;
        };
        bindings.insert(identifier, (rune_name, rune_span));
    }
}

fn binding_identifier_name(pattern: &BindingPattern<'_>) -> Option<String> {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => Some(identifier.name.as_str().to_owned()),
        _ => None,
    }
}

fn collect_exported_names(program: &Program<'_>) -> FxHashSet<String> {
    let mut names = FxHashSet::default();

    for statement in &program.body {
        let Statement::ExportNamedDeclaration(export) = statement else {
            continue;
        };

        if let Some(declaration) = &export.declaration {
            if let Declaration::VariableDeclaration(declaration) = declaration {
                for declarator in &declaration.declarations {
                    if let Some(name) = binding_identifier_name(&declarator.id) {
                        names.insert(name);
                    }
                }
            }
        }

        for specifier in &export.specifiers {
            names.insert(specifier.local.name().as_str().to_owned());
        }
    }

    names
}

fn collect_reassigned_identifiers(program: &Program<'_>) -> FxHashSet<String> {
    let mut collector = ReassignedIdentifierCollector::default();
    collector.visit_program(program);
    collector.names
}

fn add_import_records(program: &Program<'_>, target: ScriptTarget, tables: &mut AnalysisTables) {
    for statement in &program.body {
        let Statement::ImportDeclaration(declaration) = statement else {
            continue;
        };
        if declaration.import_kind.is_type() {
            continue;
        }

        let mut local_names = Vec::new();
        let mut has_runtime_specifier = false;
        if let Some(specifiers) = &declaration.specifiers {
            for specifier in specifiers {
                let name = match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                        if specifier.import_kind.is_type() {
                            continue;
                        }
                        has_runtime_specifier = true;
                        specifier.local.name.as_str()
                    }
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                        has_runtime_specifier = true;
                        specifier.local.name.as_str()
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                        has_runtime_specifier = true;
                        specifier.local.name.as_str()
                    }
                };
                local_names.push(name.to_owned());
            }
        }
        if declaration
            .specifiers
            .as_ref()
            .is_some_and(|specifiers| !specifiers.is_empty() && !has_runtime_specifier)
        {
            continue;
        }

        tables.script_imports.push(ScriptImportAnalysis {
            target,
            span: declaration.span,
            source: span_slice(program.source_text, declaration.span),
            local_names,
        });
    }
}

fn collect_runes(program: &Program<'_>) -> Vec<CollectedRune> {
    let mut collector = ScriptRuneCollector::default();
    collector.visit_program(program);
    collector.runes
}

fn collect_allowed_props_rune_spans(program: &Program<'_>) -> FxHashSet<(u32, u32)> {
    let mut spans = FxHashSet::default();
    for statement in &program.body {
        match statement {
            Statement::VariableDeclaration(declaration) => {
                collect_allowed_props_runes_in_variable_declaration(declaration, &mut spans);
            }
            Statement::ExportNamedDeclaration(export) => {
                if let Some(Declaration::VariableDeclaration(declaration)) = &export.declaration {
                    collect_allowed_props_runes_in_variable_declaration(declaration, &mut spans);
                }
            }
            _ => {}
        }
    }
    spans
}

fn collect_allowed_props_runes_in_variable_declaration(
    declaration: &VariableDeclaration<'_>,
    spans: &mut FxHashSet<(u32, u32)>,
) {
    for declarator in &declaration.declarations {
        let Some(init) = &declarator.init else {
            continue;
        };
        let expression = strip_typescript_expression_wrappers(init);
        let Expression::CallExpression(call) = expression else {
            continue;
        };
        let Some(name) = extract_rune_name(&call.callee) else {
            continue;
        };
        if name == "$props" || name == "$props.id" {
            spans.insert((call.span.start, call.span.end));
        }
    }
}

fn collect_allowed_bindable_rune_spans(
    program: &Program<'_>,
    allowed_props_spans: &FxHashSet<(u32, u32)>,
) -> FxHashSet<(u32, u32)> {
    let mut spans = FxHashSet::default();
    for statement in &program.body {
        match statement {
            Statement::VariableDeclaration(declaration) => {
                collect_allowed_bindable_runes_in_variable_declaration(
                    declaration,
                    allowed_props_spans,
                    &mut spans,
                );
            }
            Statement::ExportNamedDeclaration(export) => {
                if let Some(Declaration::VariableDeclaration(declaration)) = &export.declaration {
                    collect_allowed_bindable_runes_in_variable_declaration(
                        declaration,
                        allowed_props_spans,
                        &mut spans,
                    );
                }
            }
            _ => {}
        }
    }
    spans
}

fn collect_allowed_bindable_runes_in_variable_declaration(
    declaration: &VariableDeclaration<'_>,
    allowed_props_spans: &FxHashSet<(u32, u32)>,
    spans: &mut FxHashSet<(u32, u32)>,
) {
    for declarator in &declaration.declarations {
        let Some(init) = &declarator.init else {
            continue;
        };
        let expression = strip_typescript_expression_wrappers(init);
        let Expression::CallExpression(call) = expression else {
            continue;
        };
        if !allowed_props_spans.contains(&(call.span.start, call.span.end)) {
            continue;
        }
        collect_bindable_calls_from_binding_pattern(&declarator.id, spans);
    }
}

fn collect_bindable_calls_from_binding_pattern(
    pattern: &BindingPattern<'_>,
    spans: &mut FxHashSet<(u32, u32)>,
) {
    match pattern {
        BindingPattern::BindingIdentifier(_) => {}
        BindingPattern::ObjectPattern(pattern) => {
            for property in &pattern.properties {
                collect_bindable_calls_from_binding_pattern(&property.value, spans);
            }
            if let Some(rest) = &pattern.rest {
                collect_bindable_calls_from_binding_pattern(&rest.argument, spans);
            }
        }
        BindingPattern::ArrayPattern(pattern) => {
            for element in &pattern.elements {
                if let Some(element) = element {
                    collect_bindable_calls_from_binding_pattern(element, spans);
                }
            }
            if let Some(rest) = &pattern.rest {
                collect_bindable_calls_from_binding_pattern(&rest.argument, spans);
            }
        }
        BindingPattern::AssignmentPattern(pattern) => {
            if let Some(span) = extract_specific_rune_call_span(&pattern.right, "$bindable") {
                spans.insert((span.start, span.end));
            }
            collect_bindable_calls_from_binding_pattern(&pattern.left, spans);
        }
    }
}

fn collect_allowed_effect_rune_spans(program: &Program<'_>) -> FxHashSet<(u32, u32)> {
    let mut collector = EffectExpressionStatementCollector::default();
    collector.visit_program(program);
    collector.spans
}

fn collect_allowed_state_rune_spans(program: &Program<'_>) -> FxHashSet<(u32, u32)> {
    let mut spans = FxHashSet::default();
    for statement in &program.body {
        collect_allowed_state_runes_in_statement(statement, &mut spans);
    }
    spans
}

fn collect_allowed_state_runes_in_statement(
    statement: &Statement<'_>,
    spans: &mut FxHashSet<(u32, u32)>,
) {
    match statement {
        Statement::VariableDeclaration(declaration) => {
            collect_allowed_state_runes_in_variable_declaration(declaration, spans);
        }
        Statement::ClassDeclaration(class) => {
            collect_allowed_state_runes_in_class(class, spans);
        }
        Statement::ExportNamedDeclaration(declaration) => {
            if let Some(declaration) = &declaration.declaration {
                collect_allowed_state_runes_in_declaration(declaration, spans);
            }
        }
        Statement::ExportDefaultDeclaration(declaration) => {
            if let ExportDefaultDeclarationKind::ClassDeclaration(class) = &declaration.declaration
            {
                collect_allowed_state_runes_in_class(class, spans);
            }
        }
        _ => {}
    }
}

fn collect_allowed_state_runes_in_declaration(
    declaration: &Declaration<'_>,
    spans: &mut FxHashSet<(u32, u32)>,
) {
    match declaration {
        Declaration::VariableDeclaration(declaration) => {
            collect_allowed_state_runes_in_variable_declaration(declaration, spans);
        }
        Declaration::ClassDeclaration(class) => {
            collect_allowed_state_runes_in_class(class, spans);
        }
        _ => {}
    }
}

fn collect_allowed_state_runes_in_variable_declaration(
    declaration: &VariableDeclaration<'_>,
    spans: &mut FxHashSet<(u32, u32)>,
) {
    for declarator in &declaration.declarations {
        let Some(init) = &declarator.init else {
            continue;
        };
        if let Some(span) = extract_state_creation_rune_span(init) {
            spans.insert((span.start, span.end));
        }
    }
}

fn collect_allowed_state_runes_in_class(class: &Class<'_>, spans: &mut FxHashSet<(u32, u32)>) {
    let mut constructor = None;

    for element in &class.body.body {
        match element {
            ClassElement::PropertyDefinition(property) => {
                if property.r#static {
                    continue;
                }
                let Some(value) = &property.value else {
                    continue;
                };
                if let Some(span) = extract_state_creation_rune_span(value) {
                    spans.insert((span.start, span.end));
                }
            }
            ClassElement::MethodDefinition(method)
                if method.kind == MethodDefinitionKind::Constructor && !method.r#static =>
            {
                constructor = Some(method);
            }
            _ => {}
        }
    }

    if let Some(constructor) = constructor {
        collect_allowed_state_runes_in_constructor(constructor, spans);
    }
}

fn collect_allowed_state_runes_in_constructor(
    constructor: &MethodDefinition<'_>,
    spans: &mut FxHashSet<(u32, u32)>,
) {
    let Some(body) = &constructor.value.body else {
        return;
    };

    for statement in &body.statements {
        let Statement::ExpressionStatement(statement) = statement else {
            continue;
        };
        let Expression::AssignmentExpression(expression) = &statement.expression else {
            continue;
        };
        if expression.operator != AssignmentOperator::Assign {
            continue;
        }
        if !is_constructor_state_field_assignment_target(&expression.left) {
            continue;
        }
        if let Some(span) = extract_state_creation_rune_span(&expression.right) {
            spans.insert((span.start, span.end));
        }
    }
}

fn is_constructor_state_field_assignment_target(target: &AssignmentTarget<'_>) -> bool {
    match target {
        AssignmentTarget::StaticMemberExpression(member) => {
            matches!(
                strip_typescript_expression_wrappers(&member.object),
                Expression::ThisExpression(_)
            )
        }
        AssignmentTarget::ComputedMemberExpression(member) => {
            matches!(
                strip_typescript_expression_wrappers(&member.object),
                Expression::ThisExpression(_)
            ) && matches!(
                strip_typescript_expression_wrappers(&member.expression),
                Expression::StringLiteral(_)
                    | Expression::NumericLiteral(_)
                    | Expression::BigIntLiteral(_)
                    | Expression::BooleanLiteral(_)
                    | Expression::NullLiteral(_)
            )
        }
        AssignmentTarget::PrivateFieldExpression(member) => {
            matches!(
                strip_typescript_expression_wrappers(&member.object),
                Expression::ThisExpression(_)
            )
        }
        AssignmentTarget::TSAsExpression(target) => {
            is_constructor_state_field_assignment_expression(&target.expression)
        }
        AssignmentTarget::TSSatisfiesExpression(target) => {
            is_constructor_state_field_assignment_expression(&target.expression)
        }
        AssignmentTarget::TSNonNullExpression(target) => {
            is_constructor_state_field_assignment_expression(&target.expression)
        }
        AssignmentTarget::TSTypeAssertion(target) => {
            is_constructor_state_field_assignment_expression(&target.expression)
        }
        AssignmentTarget::AssignmentTargetIdentifier(_)
        | AssignmentTarget::ObjectAssignmentTarget(_)
        | AssignmentTarget::ArrayAssignmentTarget(_) => false,
    }
}

fn is_constructor_state_field_assignment_expression(expression: &Expression<'_>) -> bool {
    match strip_typescript_expression_wrappers(expression) {
        Expression::StaticMemberExpression(member) => {
            matches!(
                strip_typescript_expression_wrappers(&member.object),
                Expression::ThisExpression(_)
            )
        }
        Expression::ComputedMemberExpression(member) => {
            matches!(
                strip_typescript_expression_wrappers(&member.object),
                Expression::ThisExpression(_)
            ) && matches!(
                strip_typescript_expression_wrappers(&member.expression),
                Expression::StringLiteral(_)
                    | Expression::NumericLiteral(_)
                    | Expression::BigIntLiteral(_)
                    | Expression::BooleanLiteral(_)
                    | Expression::NullLiteral(_)
            )
        }
        Expression::PrivateFieldExpression(member) => {
            matches!(
                strip_typescript_expression_wrappers(&member.object),
                Expression::ThisExpression(_)
            )
        }
        _ => false,
    }
}

fn extract_state_creation_rune_span(expression: &Expression<'_>) -> Option<Span> {
    let expression = strip_typescript_expression_wrappers(expression);
    let Expression::CallExpression(call) = expression else {
        return None;
    };
    let name = extract_rune_name(&call.callee)?;
    if is_state_creation_rune(&name) {
        Some(call.span)
    } else {
        None
    }
}

fn strip_typescript_expression_wrappers<'a>(expression: &'a Expression<'a>) -> &'a Expression<'a> {
    match expression {
        Expression::TSAsExpression(wrapper) => {
            strip_typescript_expression_wrappers(&wrapper.expression)
        }
        Expression::TSSatisfiesExpression(wrapper) => {
            strip_typescript_expression_wrappers(&wrapper.expression)
        }
        Expression::TSTypeAssertion(wrapper) => {
            strip_typescript_expression_wrappers(&wrapper.expression)
        }
        Expression::TSNonNullExpression(wrapper) => {
            strip_typescript_expression_wrappers(&wrapper.expression)
        }
        Expression::TSInstantiationExpression(wrapper) => {
            strip_typescript_expression_wrappers(&wrapper.expression)
        }
        _ => expression,
    }
}

fn extract_rune_call(expression: &Expression<'_>) -> Option<(String, Span)> {
    let expression = strip_typescript_expression_wrappers(expression);
    let Expression::CallExpression(call) = expression else {
        return None;
    };
    let name = extract_rune_name(&call.callee)?;
    Some((name, call.span))
}

fn extract_specific_rune_call_span(
    expression: &Expression<'_>,
    expected_name: &str,
) -> Option<Span> {
    let (name, span) = extract_rune_call(expression)?;
    if name == expected_name {
        Some(span)
    } else {
        None
    }
}

#[derive(Debug, Clone)]
struct CollectedRune {
    name: String,
    span: oxc_span::Span,
    callee_span: oxc_span::Span,
    argument_count: u32,
}

#[derive(Default)]
struct ScriptRuneCollector {
    runes: Vec<CollectedRune>,
}

#[derive(Default)]
struct EffectExpressionStatementCollector {
    spans: FxHashSet<(u32, u32)>,
}

#[derive(Default)]
struct ReassignedIdentifierCollector {
    names: FxHashSet<String>,
}

impl<'a> Visit<'a> for ScriptRuneCollector {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Some(name) = extract_rune_name(&call.callee) {
            if name.starts_with('$') {
                self.runes.push(CollectedRune {
                    name,
                    span: call.span,
                    callee_span: call.callee.span(),
                    argument_count: call.arguments.len() as u32,
                });
            }
        }

        walk::walk_call_expression(self, call);
    }
}

impl<'a> Visit<'a> for EffectExpressionStatementCollector {
    fn visit_statement(&mut self, statement: &Statement<'a>) {
        if let Statement::ExpressionStatement(expression_statement) = statement
            && let Expression::CallExpression(call) =
                strip_typescript_expression_wrappers(&expression_statement.expression)
            && let Some(name) = extract_rune_name(&call.callee)
            && matches!(name.as_str(), "$effect" | "$effect.pre" | "$effect.root")
        {
            self.spans.insert((call.span.start, call.span.end));
        }

        walk::walk_statement(self, statement);
    }
}

impl<'a> Visit<'a> for ReassignedIdentifierCollector {
    fn visit_update_expression(&mut self, expression: &UpdateExpression<'a>) {
        collect_identifier_from_simple_target(&expression.argument, &mut self.names);
        walk::walk_update_expression(self, expression);
    }

    fn visit_assignment_target(&mut self, target: &AssignmentTarget<'a>) {
        collect_identifiers_from_assignment_target(target, &mut self.names);
        walk::walk_assignment_target(self, target);
    }
}

fn collect_identifiers_from_assignment_target(
    target: &AssignmentTarget<'_>,
    names: &mut FxHashSet<String>,
) {
    match target {
        AssignmentTarget::AssignmentTargetIdentifier(identifier) => {
            names.insert(identifier.name.as_str().to_owned());
        }
        AssignmentTarget::ObjectAssignmentTarget(target) => {
            collect_identifiers_from_object_assignment_target(target, names);
        }
        AssignmentTarget::ArrayAssignmentTarget(target) => {
            collect_identifiers_from_array_assignment_target(target, names);
        }
        AssignmentTarget::ComputedMemberExpression(_)
        | AssignmentTarget::StaticMemberExpression(_)
        | AssignmentTarget::PrivateFieldExpression(_)
        | AssignmentTarget::TSAsExpression(_)
        | AssignmentTarget::TSSatisfiesExpression(_)
        | AssignmentTarget::TSNonNullExpression(_)
        | AssignmentTarget::TSTypeAssertion(_) => {}
    }
}

fn collect_identifiers_from_object_assignment_target(
    target: &oxc_ast::ast::ObjectAssignmentTarget<'_>,
    names: &mut FxHashSet<String>,
) {
    for property in &target.properties {
        collect_identifiers_from_assignment_target_property(property, names);
    }
    if let Some(rest) = &target.rest {
        collect_identifiers_from_assignment_target(&rest.target, names);
    }
}

fn collect_identifiers_from_array_assignment_target(
    target: &oxc_ast::ast::ArrayAssignmentTarget<'_>,
    names: &mut FxHashSet<String>,
) {
    for element in &target.elements {
        if let Some(element) = element {
            collect_identifiers_from_assignment_target_maybe_default(element, names);
        }
    }
    if let Some(rest) = &target.rest {
        collect_identifiers_from_assignment_target(&rest.target, names);
    }
}

fn collect_identifiers_from_assignment_target_property(
    property: &AssignmentTargetProperty<'_>,
    names: &mut FxHashSet<String>,
) {
    match property {
        AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(property) => {
            names.insert(property.binding.name.as_str().to_owned());
        }
        AssignmentTargetProperty::AssignmentTargetPropertyProperty(property) => {
            collect_identifiers_from_assignment_target_maybe_default(&property.binding, names);
        }
    }
}

fn collect_identifiers_from_assignment_target_maybe_default(
    target: &AssignmentTargetMaybeDefault<'_>,
    names: &mut FxHashSet<String>,
) {
    match target {
        AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(identifier) => {
            names.insert(identifier.name.as_str().to_owned());
        }
        AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(target) => {
            collect_identifiers_from_assignment_target(&target.binding, names);
        }
        AssignmentTargetMaybeDefault::ArrayAssignmentTarget(target) => {
            collect_identifiers_from_array_assignment_target(target, names);
        }
        AssignmentTargetMaybeDefault::ObjectAssignmentTarget(target) => {
            collect_identifiers_from_object_assignment_target(target, names);
        }
        AssignmentTargetMaybeDefault::ComputedMemberExpression(_)
        | AssignmentTargetMaybeDefault::StaticMemberExpression(_)
        | AssignmentTargetMaybeDefault::PrivateFieldExpression(_)
        | AssignmentTargetMaybeDefault::TSAsExpression(_)
        | AssignmentTargetMaybeDefault::TSSatisfiesExpression(_)
        | AssignmentTargetMaybeDefault::TSNonNullExpression(_)
        | AssignmentTargetMaybeDefault::TSTypeAssertion(_) => {}
    }
}

fn collect_identifier_from_simple_target(
    target: &SimpleAssignmentTarget<'_>,
    names: &mut FxHashSet<String>,
) {
    if let SimpleAssignmentTarget::AssignmentTargetIdentifier(identifier) = target {
        names.insert(identifier.name.as_str().to_owned());
    }
}

fn extract_rune_name(callee: &Expression<'_>) -> Option<String> {
    match callee {
        Expression::Identifier(identifier) => Some(identifier.name.as_str().to_owned()),
        Expression::StaticMemberExpression(member) => {
            let object_name = extract_rune_name(&member.object)?;
            Some(format!("{object_name}.{}", member.property.name.as_str()))
        }
        Expression::ParenthesizedExpression(expr) => extract_rune_name(&expr.expression),
        _ => None,
    }
}

fn span_slice(source: &str, span: Span) -> String {
    let start = span.start as usize;
    let end = span.end as usize;
    if start <= end && end <= source.len() {
        source[start..end].to_owned()
    } else {
        String::new()
    }
}
