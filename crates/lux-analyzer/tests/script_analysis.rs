use lux_analyzer::analyze;
use lux_ast::analysis::{AnalysisDiagnosticCode, ScriptRuneKind, ScriptTarget};
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

#[test]
fn analyze_collects_script_rune_calls() {
    let source = r#"
<script>
  let count = $state(0);
  let doubled = $derived.by(() => count * 2);
  $effect.pre(() => {
    console.log(count);
  });
</script>
"#;

    let tables = analyze_source(source);
    let runes: Vec<_> = tables
        .script_runes
        .iter()
        .filter(|rune| rune.target == ScriptTarget::Instance)
        .collect();

    assert!(runes.iter().any(|rune| rune.name == "$state"
        && rune.kind == ScriptRuneKind::Known
        && rune.is_state_creation));
    assert!(runes.iter().any(|rune| rune.name == "$derived.by"
        && rune.kind == ScriptRuneKind::Known
        && rune.is_state_creation));
    assert!(runes.iter().any(|rune| rune.name == "$effect.pre"
        && rune.kind == ScriptRuneKind::Known
        && !rune.is_state_creation));
}

#[test]
fn analyze_marks_unknown_runes() {
    let source = r#"
<script>
  $unknown_rune(1, 2, 3);
</script>
"#;

    let tables = analyze_source(source);
    assert!(tables.script_runes.iter().any(|rune| {
        rune.target == ScriptTarget::Instance
            && rune.name == "$unknown_rune"
            && rune.kind == ScriptRuneKind::Unknown
            && rune.argument_count == 3
    }));
}

#[test]
fn analyze_collects_script_imports() {
    let source = r#"
<script context="module">
  import { m } from './m';
</script>
<script>
  import * as Tabs from './tabs';
  import x, { y as z } from "./x";
</script>
"#;

    let tables = analyze_source(source);

    assert!(tables.script_imports.iter().any(|item| {
        item.target == ScriptTarget::Module
            && item.source.contains("import { m } from './m';")
            && item.local_names == vec!["m".to_string()]
    }));
    assert!(tables.script_imports.iter().any(|item| {
        item.target == ScriptTarget::Instance
            && item.source.contains("import * as Tabs from './tabs';")
            && item.local_names == vec!["Tabs".to_string()]
    }));
    assert!(tables.script_imports.iter().any(|item| {
        item.target == ScriptTarget::Instance
            && item.source.contains("import x, { y as z } from \"./x\";")
            && item.local_names == vec!["x".to_string(), "z".to_string()]
    }));
}

#[test]
fn analyze_skips_type_only_imports_and_type_specifiers() {
    let source = r#"
<script lang="ts">
  import type { A } from './types';
  import { type B, c } from './mixed';
  import {} from './side-effect';
</script>
"#;

    let tables = analyze_source(source);
    let instance_imports: Vec<_> = tables
        .script_imports
        .iter()
        .filter(|item| item.target == ScriptTarget::Instance)
        .collect();

    assert_eq!(instance_imports.len(), 2);
    assert!(instance_imports.iter().any(|item| {
        item.source.contains("import { type B, c } from './mixed';")
            && item.local_names == vec!["c".to_string()]
    }));
    assert!(instance_imports.iter().any(|item| {
        item.source.contains("import {} from './side-effect';") && item.local_names.is_empty()
    }));
    assert!(
        !instance_imports
            .iter()
            .any(|item| item.source.contains("import type { A } from './types';"))
    );
}

#[test]
fn analyze_reports_rune_argument_diagnostics() {
    let source = r#"
<script>
  const derived = $derived();
  const props = $props(1);
  $inspect.trace('a', 'b');
  $inspect('a').with();
  $state.eager();
</script>
"#;

    let tables = analyze_source(source);
    assert!(tables.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == AnalysisDiagnosticCode::ScriptRuneInvalidArgumentsLength
    }));
    assert!(tables
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == AnalysisDiagnosticCode::ScriptRuneInvalidArguments));
}

#[test]
fn analyze_reports_props_bindable_and_effect_placement() {
    let source = r#"
<script>
  if (ok) {
    const props = $props();
  }
  const bindable = $bindable();
  const effect_value = $effect(() => {});
</script>
"#;

    let tables = analyze_source(source);
    let invalid_placement_count = tables
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.code == AnalysisDiagnosticCode::TemplateRuneInvalidPlacement
        })
        .count();
    assert!(
        invalid_placement_count >= 3,
        "expected at least 3 invalid placement diagnostics, got {}",
        invalid_placement_count
    );
}

#[test]
fn analyze_reports_props_rune_invalid_placement_in_module_script() {
    let source = r#"
<script context="module">
  const props = $props();
</script>
"#;

    let tables = analyze_source(source);
    assert!(tables.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == AnalysisDiagnosticCode::TemplateRuneInvalidPlacement
            && diagnostic.message.contains("$props()")
    }));
}

#[test]
fn analyze_reports_props_id_invalid_placement_for_destructuring() {
    let source = r#"
<script>
  const { id } = $props.id();
</script>
"#;

    let tables = analyze_source(source);
    assert!(tables.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == AnalysisDiagnosticCode::PropsIdInvalidPlacement
            && diagnostic.message.contains("$props.id()")
    }));
}

#[test]
fn analyze_reports_props_duplicate() {
    let source = r#"
<script>
  const props = $props();
  const more = $props();
</script>
"#;

    let tables = analyze_source(source);
    assert!(tables.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == AnalysisDiagnosticCode::PropsDuplicate
    }));
}

#[test]
fn analyze_reports_props_invalid_identifier_and_pattern() {
    let source = r#"
<script>
  const [value] = $props();
  const { [key]: value, nested: { inner } } = $props();
</script>
"#;

    let tables = analyze_source(source);
    assert!(tables.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == AnalysisDiagnosticCode::PropsInvalidIdentifier
    }));
    assert!(tables.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == AnalysisDiagnosticCode::PropsInvalidPattern
    }));
}

#[test]
fn analyze_reports_rune_invalid_spread() {
    let source = r#"
<script>
  const args = [0];
  const value = $state(...args);
</script>
"#;

    let tables = analyze_source(source);
    assert!(tables.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == AnalysisDiagnosticCode::ScriptRuneInvalidSpread
    }));
}

#[test]
fn analyze_reports_inspect_trace_placement_and_generator() {
    let source = r#"
<script>
  $inspect.trace();
  function nope() {
    let x = 1;
    $inspect.trace();
  }
  function* gen() {
    $inspect.trace();
  }
</script>
"#;

    let tables = analyze_source(source);
    assert!(tables.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == AnalysisDiagnosticCode::InspectTraceInvalidPlacement
    }));
    assert!(tables.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == AnalysisDiagnosticCode::InspectTraceGenerator
    }));
}

#[test]
fn analyze_reports_duplicate_class_field() {
    let source = r#"
<script>
  class Counter {
    count = 0;
    count = $state(1);
  }
</script>
"#;

    let tables = analyze_source(source);
    assert!(
        tables
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == AnalysisDiagnosticCode::DuplicateClassField })
    );
}

#[test]
fn analyze_reports_state_field_duplicate() {
    let source = r#"
<script>
  class Counter {
    count = $state(0);
    count = $state(1);
  }
</script>
"#;

    let tables = analyze_source(source);
    assert!(
        tables
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == AnalysisDiagnosticCode::StateFieldDuplicate })
    );
}

#[test]
fn analyze_reports_state_field_invalid_assignment_before_constructor_declaration() {
    let source = r#"
<script>
  class Counter {
    constructor() {
      this.count = 1;
      this.count = $state(0);
    }
  }
</script>
"#;

    let tables = analyze_source(source);
    assert!(tables.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == AnalysisDiagnosticCode::StateFieldInvalidAssignment
    }));
}

#[test]
fn analyze_allows_state_rune_in_constructor_root_assignment() {
    let source = r#"
<script>
  class Counter {
    constructor() {
      this.count = $state(0);
      this.label = $derived.by(() => this.count + 1);
    }
  }
</script>
"#;

    let tables = analyze_source(source);
    assert!(!tables.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == AnalysisDiagnosticCode::TemplateRuneInvalidPlacement
    }));
}

#[test]
fn analyze_rejects_state_rune_in_nested_constructor_assignment() {
    let source = r#"
<script>
  class Counter {
    constructor() {
      if (ok) {
        this.count = $state(0);
      }
    }
  }
</script>
"#;

    let tables = analyze_source(source);
    assert!(tables.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == AnalysisDiagnosticCode::TemplateRuneInvalidPlacement
    }));
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
