use lux_analyzer::analyze;
use lux_ast::analysis::{AnalysisTables, TemplateBindingKind, TemplateScopeId, TemplateScopeKind};
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
