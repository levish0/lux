use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use lux_analyzer::analyze;
use lux_ast::analysis::{AnalysisDiagnosticCode, AnalysisSeverity};
use lux_parser::parse;
use oxc_allocator::Allocator;
use serde_json::Value;

#[derive(Clone, Copy)]
struct ParityCase<'a> {
    name: &'a str,
    source: &'a str,
    reference_code: &'a str,
    lux_code: AnalysisDiagnosticCode,
    severity: AnalysisSeverity,
}

#[test]
fn parity_against_reference_analyzer_diagnostics_smoke() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("failed to resolve workspace root");
    let runner_dir = ensure_svelte_runner(workspace_root);

    let generated_dir = manifest_dir.join("target/parity-reference-diagnostics");
    let _ = fs::create_dir_all(&generated_dir);

    let cases = vec![
        ParityCase {
            name: "bind_invalid_expression",
            source: "<input bind:value={count + 1} />",
            reference_code: "bind_invalid_expression",
            lux_code: AnalysisDiagnosticCode::BindDirectiveInvalidExpression,
            severity: AnalysisSeverity::Error,
        },
        ParityCase {
            name: "bind_invalid_name",
            source: "<div bind:notARealBinding={value} />",
            reference_code: "bind_invalid_name",
            lux_code: AnalysisDiagnosticCode::BindDirectiveUnknownName,
            severity: AnalysisSeverity::Error,
        },
        ParityCase {
            name: "bind_invalid_target_window",
            source: "<div bind:innerWidth={width} />",
            reference_code: "bind_invalid_target",
            lux_code: AnalysisDiagnosticCode::BindDirectiveInvalidTarget,
            severity: AnalysisSeverity::Error,
        },
        ParityCase {
            name: "bind_group_invalid_expression",
            source: "<input bind:group={get(), set(value)} />",
            reference_code: "bind_group_invalid_expression",
            lux_code: AnalysisDiagnosticCode::BindDirectiveGroupInvalidExpression,
            severity: AnalysisSeverity::Error,
        },
        ParityCase {
            name: "attribute_invalid_multiple",
            source: "<select multiple={is_many} bind:value={selected}></select>",
            reference_code: "attribute_invalid_multiple",
            lux_code: AnalysisDiagnosticCode::BindDirectiveSelectMultipleDynamic,
            severity: AnalysisSeverity::Error,
        },
        ParityCase {
            name: "attribute_contenteditable_missing",
            source: "<div bind:textContent={content}></div>",
            reference_code: "attribute_contenteditable_missing",
            lux_code: AnalysisDiagnosticCode::BindDirectiveContenteditableMissing,
            severity: AnalysisSeverity::Error,
        },
        ParityCase {
            name: "attribute_contenteditable_dynamic",
            source: "<div contenteditable={is_editable} bind:textContent={content}></div>",
            reference_code: "attribute_contenteditable_dynamic",
            lux_code: AnalysisDiagnosticCode::BindDirectiveContenteditableDynamic,
            severity: AnalysisSeverity::Error,
        },
        ParityCase {
            name: "render_tag_invalid_spread_argument",
            source: "{@render snippet(...args)}",
            reference_code: "render_tag_invalid_spread_argument",
            lux_code: AnalysisDiagnosticCode::RenderTagInvalidSpreadArgument,
            severity: AnalysisSeverity::Error,
        },
        ParityCase {
            name: "render_tag_invalid_call_expression",
            source: "{@render snippet.call(ctx)}",
            reference_code: "render_tag_invalid_call_expression",
            lux_code: AnalysisDiagnosticCode::RenderTagInvalidCallExpression,
            severity: AnalysisSeverity::Error,
        },
        ParityCase {
            name: "let_directive_invalid_placement",
            source: "<svelte:window let:item />",
            reference_code: "let_directive_invalid_placement",
            lux_code: AnalysisDiagnosticCode::LetDirectiveInvalidPlacement,
            severity: AnalysisSeverity::Error,
        },
        ParityCase {
            name: "snippet_invalid_rest_parameter",
            source: "{#snippet demo(...args)}{/snippet}",
            reference_code: "snippet_invalid_rest_parameter",
            lux_code: AnalysisDiagnosticCode::SnippetInvalidRestParameter,
            severity: AnalysisSeverity::Error,
        },
        ParityCase {
            name: "snippet_duplicate",
            source: "{#snippet demo()}{/snippet}{#snippet demo()}{/snippet}",
            reference_code: "declaration_duplicate",
            lux_code: AnalysisDiagnosticCode::SnippetDuplicateName,
            severity: AnalysisSeverity::Error,
        },
        ParityCase {
            name: "block_empty",
            source: "{#snippet empty()} {/snippet}",
            reference_code: "block_empty",
            lux_code: AnalysisDiagnosticCode::BlockEmpty,
            severity: AnalysisSeverity::Warning,
        },
        ParityCase {
            name: "state_invalid_placement",
            source: "{$state(1)}",
            reference_code: "state_invalid_placement",
            lux_code: AnalysisDiagnosticCode::TemplateRuneInvalidPlacement,
            severity: AnalysisSeverity::Error,
        },
        ParityCase {
            name: "window_invalid_placement",
            source: "<div><svelte:window /></div>",
            reference_code: "svelte_meta_invalid_placement",
            lux_code: AnalysisDiagnosticCode::SvelteMetaInvalidPlacement,
            severity: AnalysisSeverity::Error,
        },
        ParityCase {
            name: "window_invalid_content",
            source: "<svelte:window><div /></svelte:window>",
            reference_code: "svelte_meta_invalid_content",
            lux_code: AnalysisDiagnosticCode::SvelteMetaInvalidContent,
            severity: AnalysisSeverity::Error,
        },
        ParityCase {
            name: "window_duplicate",
            source: "<svelte:window /><svelte:window />",
            reference_code: "svelte_meta_duplicate",
            lux_code: AnalysisDiagnosticCode::SvelteMetaDuplicate,
            severity: AnalysisSeverity::Error,
        },
    ];

    for case in cases {
        let input_path = generated_dir.join(format!("{}.svelte", case.name));
        let output_path = generated_dir.join(format!("{}.reference.json", case.name));
        fs::write(&input_path, case.source).unwrap_or_else(|error| {
            panic!("failed to write {}: {error}", input_path.display());
        });

        let reference = run_reference_analyze(&runner_dir, &input_path, &output_path);
        let reference_codes = match case.severity {
            AnalysisSeverity::Error => reference_codes(&reference, "errors"),
            AnalysisSeverity::Warning => reference_codes(&reference, "warnings"),
        };

        assert!(
            reference_codes
                .iter()
                .any(|code| code == case.reference_code),
            "reference missing expected code `{}` for `{}`; got {:?}",
            case.reference_code,
            case.name,
            reference_codes
        );

        let allocator = Allocator::default();
        let parsed = parse(case.source, &allocator, false);
        assert!(
            parsed.errors.is_empty(),
            "parse failed for `{}` with {} errors",
            case.name,
            parsed.errors.len()
        );

        let tables = analyze(&parsed.root);
        let lux_codes: Vec<_> = tables
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == case.severity)
            .map(|diagnostic| diagnostic.code)
            .collect();

        assert!(
            lux_codes.iter().any(|code| *code == case.lux_code),
            "lux missing expected diagnostic {:?} for `{}`; got {:?}",
            case.lux_code,
            case.name,
            lux_codes
        );
    }
}

fn npm_executable() -> &'static str {
    if cfg!(windows) { "npm.cmd" } else { "npm" }
}

fn node_executable() -> &'static str {
    if cfg!(windows) { "node.exe" } else { "node" }
}

fn ensure_svelte_runner(workspace_root: &Path) -> PathBuf {
    let runner_dir = workspace_root.join("tools/svelte_runner");
    let script_path = runner_dir.join("analyze_diagnostics.mjs");
    assert!(script_path.exists(), "missing {}", script_path.display());

    let svelte_module = runner_dir.join("node_modules/svelte/package.json");
    if svelte_module.exists() {
        return runner_dir;
    }

    let install = Command::new(npm_executable())
        .arg("install")
        .arg("--silent")
        .arg("--no-fund")
        .arg("--no-audit")
        .current_dir(&runner_dir)
        .output()
        .unwrap_or_else(|error| {
            panic!(
                "failed to run npm install in {}: {error}",
                runner_dir.display()
            )
        });

    assert!(
        install.status.success(),
        "npm install failed in {}\nstdout:\n{}\nstderr:\n{}",
        runner_dir.display(),
        String::from_utf8_lossy(&install.stdout),
        String::from_utf8_lossy(&install.stderr),
    );

    runner_dir
}

fn run_reference_analyze(runner_dir: &Path, input_path: &Path, output_path: &Path) -> Value {
    let script_path = runner_dir.join("analyze_diagnostics.mjs");
    let run = Command::new(node_executable())
        .arg(&script_path)
        .arg(input_path)
        .arg(output_path)
        .current_dir(runner_dir)
        .output()
        .unwrap_or_else(|error| panic!("failed to run node for {}: {error}", input_path.display()));

    assert!(
        run.status.success(),
        "reference analyze failed for {}\nstdout:\n{}\nstderr:\n{}",
        input_path.display(),
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr),
    );

    let raw = fs::read_to_string(output_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", output_path.display()));
    serde_json::from_str(&raw)
        .unwrap_or_else(|error| panic!("invalid JSON {}: {error}", output_path.display()))
}

fn reference_codes(reference: &Value, key: &str) -> Vec<String> {
    reference
        .get(key)
        .and_then(Value::as_array)
        .map(|entries| {
            entries
                .iter()
                .filter_map(|entry| entry.get("code").and_then(Value::as_str))
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}
