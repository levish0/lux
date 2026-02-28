use lux_analyzer::analyze;
use lux_ast::analysis::ScriptTarget;
use lux_parser::parse;
use oxc_allocator::Allocator;

#[test]
fn analyze_collects_script_scopes_symbols_and_references() {
    let source = r#"
<script context="module">
  export const modValue = 1;
  let moduleCount = 0;
  moduleCount += 1;
</script>
<script>
  let count = 0;
  function bump() {
    count += 1;
  }
  count = count + 1;
</script>
"#;

    let tables = analyze_source(source);

    assert!(
        tables
            .script_scopes
            .iter()
            .any(|scope| scope.target == ScriptTarget::Module)
    );
    assert!(
        tables
            .script_scopes
            .iter()
            .any(|scope| scope.target == ScriptTarget::Instance)
    );

    assert!(
        tables.script_symbols.iter().any(|symbol| {
            symbol.target == ScriptTarget::Module && symbol.name == "moduleCount"
        })
    );
    assert!(
        tables
            .script_symbols
            .iter()
            .any(|symbol| symbol.target == ScriptTarget::Instance && symbol.name == "count")
    );

    assert!(
        tables
            .script_references
            .iter()
            .any(|reference| reference.target == ScriptTarget::Module
                && reference.name == "moduleCount")
    );
    assert!(
        tables.script_references.iter().any(
            |reference| reference.target == ScriptTarget::Instance && reference.name == "count"
        )
    );
}

#[test]
fn analyze_script_reference_read_write_flags() {
    let source = r#"
<script>
  let count = 0;
  count = count + 1;
  count += 1;
  ++count;
</script>
"#;

    let tables = analyze_source(source);

    let count_references: Vec<_> = tables
        .script_references
        .iter()
        .filter(|reference| reference.target == ScriptTarget::Instance && reference.name == "count")
        .collect();

    assert!(count_references.iter().any(|reference| reference.is_write));
    assert!(count_references.iter().any(|reference| reference.is_read));
    assert!(
        count_references
            .iter()
            .any(|reference| reference.is_read && reference.is_write)
    );
}

fn analyze_source(source: &str) -> lux_ast::analysis::AnalysisTables {
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(
        parsed.errors.is_empty(),
        "parse failed with {} errors",
        parsed.errors.len()
    );

    analyze(&parsed.root)
}
