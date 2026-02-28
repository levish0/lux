use lux_analyzer::analyze;
use lux_ast::analysis::{
    AnalysisDiagnosticCode, AnalysisSeverity, AnalysisTables, TemplateBindingKind, TemplateScopeId,
    TemplateScopeKind,
};
use lux_parser::parse;
use oxc_allocator::Allocator;

#[test]
fn analyze_collects_each_context_and_index_bindings() {
    let tables = analyze_source("{#each items as { id, name = 'x' }, i}{id}{i}{/each}");

    let mut context_names: Vec<String> = tables
        .template_bindings
        .iter()
        .filter(|binding| binding.kind == TemplateBindingKind::EachContext)
        .map(|binding| binding.name.clone())
        .collect();
    context_names.sort();
    assert_eq!(context_names, vec!["id", "name"]);

    let index_bindings: Vec<_> = tables
        .template_bindings
        .iter()
        .filter(|binding| binding.kind == TemplateBindingKind::EachIndex)
        .collect();
    assert_eq!(index_bindings.len(), 1);
    assert_eq!(index_bindings[0].name, "i");

    for binding in tables.template_bindings.iter().filter(|binding| {
        matches!(
            binding.kind,
            TemplateBindingKind::EachContext | TemplateBindingKind::EachIndex
        )
    }) {
        assert_eq!(
            scope_kind_for(&tables, binding.scope),
            TemplateScopeKind::Each
        );
    }
}

#[test]
fn analyze_collects_await_then_and_catch_bindings() {
    let tables = analyze_source(
        "{#await promise then { value, nested: { inner } }}{value}{inner}{:catch error}{error}{/await}",
    );

    let mut value_names: Vec<String> = tables
        .template_bindings
        .iter()
        .filter(|binding| binding.kind == TemplateBindingKind::AwaitValue)
        .map(|binding| binding.name.clone())
        .collect();
    value_names.sort();
    assert_eq!(value_names, vec!["inner", "value"]);

    let error_bindings: Vec<_> = tables
        .template_bindings
        .iter()
        .filter(|binding| binding.kind == TemplateBindingKind::AwaitError)
        .collect();
    assert_eq!(error_bindings.len(), 1);
    assert_eq!(error_bindings[0].name, "error");

    for binding in tables
        .template_bindings
        .iter()
        .filter(|binding| binding.kind == TemplateBindingKind::AwaitValue)
    {
        assert_eq!(
            scope_kind_for(&tables, binding.scope),
            TemplateScopeKind::AwaitThen
        );
    }

    for binding in error_bindings {
        assert_eq!(
            scope_kind_for(&tables, binding.scope),
            TemplateScopeKind::AwaitCatch
        );
    }
}

#[test]
fn analyze_collects_snippet_name_and_parameter_bindings() {
    let tables = analyze_source("{#snippet demo({ id }, count = 1)}{/snippet}");

    let snippet_name = tables
        .template_bindings
        .iter()
        .find(|binding| binding.kind == TemplateBindingKind::SnippetName)
        .expect("expected snippet name binding");
    assert_eq!(snippet_name.name, "demo");
    assert_eq!(
        scope_kind_for(&tables, snippet_name.scope),
        TemplateScopeKind::Root
    );

    let mut parameter_names: Vec<String> = tables
        .template_bindings
        .iter()
        .filter(|binding| binding.kind == TemplateBindingKind::SnippetParameter)
        .map(|binding| binding.name.clone())
        .collect();
    parameter_names.sort();
    assert_eq!(parameter_names, vec!["count", "id"]);

    for binding in tables
        .template_bindings
        .iter()
        .filter(|binding| binding.kind == TemplateBindingKind::SnippetParameter)
    {
        assert_eq!(
            scope_kind_for(&tables, binding.scope),
            TemplateScopeKind::Snippet
        );
    }
}

#[test]
fn analyze_collects_let_directive_bindings() {
    let tables = analyze_source(
        "<Comp let:item let:data={{ a, b: renamed }}><p>{item}{a}{renamed}</p></Comp>",
    );

    let mut let_names: Vec<String> = tables
        .template_bindings
        .iter()
        .filter(|binding| binding.kind == TemplateBindingKind::LetDirective)
        .map(|binding| binding.name.clone())
        .collect();
    let_names.sort();
    assert_eq!(let_names, vec!["a", "item", "renamed"]);

    for binding in tables
        .template_bindings
        .iter()
        .filter(|binding| binding.kind == TemplateBindingKind::LetDirective)
    {
        assert_eq!(
            scope_kind_for(&tables, binding.scope),
            TemplateScopeKind::Element
        );
    }
}

#[test]
fn analyze_collects_template_reference_read_write_flags() {
    let tables = analyze_source("{value}{value = value + 1}{++count}");

    let value_refs: Vec<_> = tables
        .template_references
        .iter()
        .filter(|reference| reference.name == "value")
        .collect();
    assert!(value_refs.iter().any(|reference| reference.is_read));
    assert!(value_refs.iter().any(|reference| reference.is_write));

    let count_refs: Vec<_> = tables
        .template_references
        .iter()
        .filter(|reference| reference.name == "count")
        .collect();
    assert!(count_refs.iter().any(|reference| reference.is_read));
    assert!(count_refs.iter().any(|reference| reference.is_write));
}

#[test]
fn analyze_reports_bind_invalid_expression() {
    let tables = analyze_source("<input bind:value={count + 1} />");

    assert!(tables.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == AnalysisDiagnosticCode::BindDirectiveInvalidExpression
            && diagnostic.severity == AnalysisSeverity::Error
    }));
}

#[test]
fn analyze_reports_snippet_duplicate_name() {
    let tables = analyze_source("{#snippet demo()}{/snippet}{#snippet demo()}{/snippet}");

    assert!(tables.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == AnalysisDiagnosticCode::SnippetDuplicateName
            && diagnostic.severity == AnalysisSeverity::Error
    }));
}

#[test]
fn analyze_reports_assignment_to_template_binding() {
    let tables = analyze_source("{#each items as item}{item = 1}{/each}");

    assert!(tables.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == AnalysisDiagnosticCode::TemplateAssignmentToBinding
            && diagnostic.severity == AnalysisSeverity::Error
    }));
}

#[test]
fn analyze_reports_assignment_to_const_and_import() {
    let tables = analyze_source(
        "<script>import { value as imported } from 'pkg'; const fixed = 1;</script>{fixed = 2}{imported = 3}",
    );

    assert!(tables.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == AnalysisDiagnosticCode::TemplateAssignmentToConst
            && diagnostic.severity == AnalysisSeverity::Error
    }));
    assert!(tables.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == AnalysisDiagnosticCode::TemplateAssignmentToImport
            && diagnostic.severity == AnalysisSeverity::Error
    }));
}

#[test]
fn analyze_reports_block_empty_warning() {
    let tables = analyze_source("{#snippet empty()} {/snippet}");

    assert!(tables.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == AnalysisDiagnosticCode::BlockEmpty
            && diagnostic.severity == AnalysisSeverity::Warning
    }));
}

#[test]
fn analyze_treats_bind_identifier_as_read_and_write_reference() {
    let tables = analyze_source("<input bind:value={count} />");

    let count_refs: Vec<_> = tables
        .template_references
        .iter()
        .filter(|reference| reference.name == "count")
        .collect();

    assert!(count_refs.iter().any(|reference| reference.is_read));
    assert!(count_refs.iter().any(|reference| reference.is_write));
}

#[test]
fn analyze_reports_bind_assignment_to_const_and_import() {
    let tables = analyze_source(
        "<script>import { value as imported } from 'pkg'; const fixed = 1; let ok = 0;</script><input bind:value={fixed} /><input bind:value={imported} /><input bind:value={ok} />",
    );

    assert!(tables.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == AnalysisDiagnosticCode::TemplateAssignmentToConst
            && diagnostic.severity == AnalysisSeverity::Error
    }));
    assert!(tables.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == AnalysisDiagnosticCode::TemplateAssignmentToImport
            && diagnostic.severity == AnalysisSeverity::Error
    }));
}

#[test]
fn analyze_ignores_module_script_const_for_template_assignment_checks() {
    let tables = analyze_source(
        "<script context=\"module\">const count = 1;</script><script>let count = 0;</script>{count = 2}",
    );

    assert!(!tables.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == AnalysisDiagnosticCode::TemplateAssignmentToConst
            || diagnostic.code == AnalysisDiagnosticCode::TemplateAssignmentToImport
    }));
}

#[test]
fn analyze_reports_bind_unknown_name_on_regular_element() {
    let tables = analyze_source("<div bind:notARealBinding={value} />");

    assert!(tables.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == AnalysisDiagnosticCode::BindDirectiveUnknownName
            && diagnostic.severity == AnalysisSeverity::Error
    }));
}

#[test]
fn analyze_reports_bind_invalid_target_for_window_binding_on_regular_element() {
    let tables = analyze_source("<div bind:innerWidth={width} />");

    assert!(tables.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == AnalysisDiagnosticCode::BindDirectiveInvalidTarget
            && diagnostic.severity == AnalysisSeverity::Error
    }));
}

#[test]
fn analyze_accepts_window_binding_on_svelte_window() {
    let tables = analyze_source("<svelte:window bind:innerWidth={width} />");

    assert!(!tables.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == AnalysisDiagnosticCode::BindDirectiveInvalidTarget
            || diagnostic.code == AnalysisDiagnosticCode::BindDirectiveUnknownName
    }));
}

#[test]
fn analyze_reports_bind_group_getter_setter_expression() {
    let tables = analyze_source("<input bind:group={get(), set(value)} />");

    assert!(tables.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == AnalysisDiagnosticCode::BindDirectiveGroupInvalidExpression
            && diagnostic.severity == AnalysisSeverity::Error
    }));
}

#[test]
fn analyze_reports_bind_checked_type_mismatch() {
    let tables = analyze_source("<input type=\"text\" bind:checked={checked} />");

    assert!(tables.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == AnalysisDiagnosticCode::BindDirectiveInputTypeMismatch
            && diagnostic.severity == AnalysisSeverity::Error
    }));
}

#[test]
fn analyze_reports_bind_files_type_mismatch_without_type_attribute() {
    let tables = analyze_source("<input bind:files={files} />");

    assert!(tables.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == AnalysisDiagnosticCode::BindDirectiveInputTypeMismatch
            && diagnostic.severity == AnalysisSeverity::Error
    }));
}

#[test]
fn analyze_reports_dynamic_input_type_for_bind_checked() {
    let tables = analyze_source("<input type={field_type} bind:checked={checked} />");

    assert!(tables.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == AnalysisDiagnosticCode::BindDirectiveInputTypeInvalid
            && diagnostic.severity == AnalysisSeverity::Error
    }));
}

#[test]
fn analyze_reports_dynamic_select_multiple_attribute_with_bind() {
    let tables = analyze_source("<select multiple={is_many} bind:value={selected}></select>");

    assert!(tables.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == AnalysisDiagnosticCode::BindDirectiveSelectMultipleDynamic
            && diagnostic.severity == AnalysisSeverity::Error
    }));
}

#[test]
fn analyze_reports_contenteditable_missing_for_text_content_binding() {
    let tables = analyze_source("<div bind:textContent={content}></div>");

    assert!(tables.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == AnalysisDiagnosticCode::BindDirectiveContenteditableMissing
            && diagnostic.severity == AnalysisSeverity::Error
    }));
}

#[test]
fn analyze_reports_dynamic_contenteditable_attribute_for_text_content_binding() {
    let tables =
        analyze_source("<div contenteditable={is_editable} bind:textContent={content}></div>");

    assert!(tables.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == AnalysisDiagnosticCode::BindDirectiveContenteditableDynamic
            && diagnostic.severity == AnalysisSeverity::Error
    }));
}

#[test]
fn analyze_reports_block_empty_warning_for_if_and_key_blocks() {
    let tables = analyze_source("{#if cond} {/if}{#key value} {/key}");

    let empty_block_count = tables
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.code == AnalysisDiagnosticCode::BlockEmpty
                && diagnostic.severity == AnalysisSeverity::Warning
        })
        .count();

    assert_eq!(empty_block_count, 2);
}

#[test]
fn analyze_reports_invalid_let_directive_placement() {
    let tables = analyze_source("<svelte:window let:item />");

    assert!(tables.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == AnalysisDiagnosticCode::LetDirectiveInvalidPlacement
            && diagnostic.severity == AnalysisSeverity::Error
    }));
}

fn analyze_source(source: &str) -> AnalysisTables {
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(
        parsed.errors.is_empty(),
        "parse failed with {} errors",
        parsed.errors.len()
    );

    analyze(&parsed.root)
}

fn scope_kind_for(tables: &AnalysisTables, scope_id: TemplateScopeId) -> TemplateScopeKind {
    tables
        .template_scopes
        .iter()
        .find(|scope| scope.id == scope_id)
        .map(|scope| scope.kind)
        .expect("missing scope for binding")
}
