use rustc_hash::FxHashSet;

use lux_ast::analysis::{
    AnalysisDiagnostic, AnalysisDiagnosticCode, AnalysisSeverity, AnalysisTables,
    ScriptImportAnalysis, ScriptReferenceAnalysis, ScriptRuneAnalysis, ScriptRuneKind,
    ScriptScopeAnalysis, ScriptSymbolAnalysis, ScriptTarget,
};
use lux_utils::runes::{is_rune, is_state_creation_rune};
use oxc_ast::AstKind;
use oxc_ast::ast::{
    CallExpression, Class, ClassElement, Declaration, ExportDefaultDeclarationKind, Expression,
    ImportDeclarationSpecifier, Program, Statement, VariableDeclaration,
};
use oxc_ast_visit::{Visit, walk};
use oxc_semantic::{ReferenceId, Semantic, SemanticBuilder};
use oxc_span::{GetSpan, Span};

pub(super) fn analyze_program(
    program: &Program<'_>,
    target: ScriptTarget,
    tables: &mut AnalysisTables,
) {
    let semantic_result = SemanticBuilder::new().build(program);
    let semantic = semantic_result.semantic;

    add_scope_records(&semantic, target, tables);
    add_symbol_records(&semantic, target, tables);
    add_reference_records(&semantic, target, tables);
    add_rune_records(program, target, tables);
    add_rune_placement_diagnostics(program, target, tables);
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

fn add_rune_records(program: &Program<'_>, target: ScriptTarget, tables: &mut AnalysisTables) {
    let mut collector = ScriptRuneCollector::default();
    collector.visit_program(program);

    for rune in collector.runes {
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

fn add_rune_placement_diagnostics(
    program: &Program<'_>,
    _target: ScriptTarget,
    tables: &mut AnalysisTables,
) {
    let allowed_spans = collect_allowed_state_rune_spans(program);
    let mut collector = ScriptRuneCollector::default();
    collector.visit_program(program);

    for rune in collector.runes {
        if !is_state_creation_rune(&rune.name) {
            continue;
        }
        if allowed_spans.contains(&(rune.span.start, rune.span.end)) {
            continue;
        }

        tables.diagnostics.push(AnalysisDiagnostic {
            severity: AnalysisSeverity::Error,
            code: AnalysisDiagnosticCode::TemplateRuneInvalidPlacement,
            message: format!(
                "`{}(...)` can only be used as a variable declaration initializer, a class field declaration, or the first assignment to a class field at the top level of the constructor.",
                rune.name
            ),
            span: rune.callee_span,
        });
    }
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
            if let ExportDefaultDeclarationKind::ClassDeclaration(class) = &declaration.declaration {
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
    for element in &class.body.body {
        let ClassElement::PropertyDefinition(property) = element else {
            continue;
        };
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
        Expression::TSAsExpression(wrapper) => strip_typescript_expression_wrappers(&wrapper.expression),
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
