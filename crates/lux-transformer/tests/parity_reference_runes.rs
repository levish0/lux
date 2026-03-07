use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use lux_analyzer::analyze;
use lux_parser::parse;
use lux_test_support::{ensure_svelte_runner, node_executable, workspace_root_from_manifest_dir};
use lux_transformer::transform;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use serde_json::Value;

#[derive(Clone, Copy)]
struct ParityCase<'a> {
    name: &'a str,
    source: &'a str,
}

#[test]
fn parity_reference_rune_lowering_smoke() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = workspace_root_from_manifest_dir(&manifest_dir);
    let runner_dir = ensure_svelte_runner(&workspace_root);
    let generated_dir = manifest_dir.join("target/parity-reference-transform-runes");
    let _ = fs::create_dir_all(&generated_dir);

    let cases = vec![
        ParityCase {
            name: "state_and_derived",
            source: "<script>let count = $state(1); let doubled = $derived.by(() => count * 2);</script>{doubled}",
        },
        ParityCase {
            name: "props_and_bindable",
            source: "<script>let { value = $bindable() } = $props();</script>{value}",
        },
        ParityCase {
            name: "props_id",
            source: "<script>const id = $props.id();</script><div id={id}></div>",
        },
        ParityCase {
            name: "state_snapshot",
            source: "<script>const snapshot = $state.snapshot(source);</script>{snapshot}",
        },
    ];

    for case in cases {
        let input_path = generated_dir.join(format!("{}.svelte", case.name));
        let output_path = generated_dir.join(format!("{}.reference.json", case.name));
        fs::write(&input_path, case.source)
            .unwrap_or_else(|error| panic!("failed to write {}: {error}", input_path.display()));

        let reference_js = run_reference_compile(&runner_dir, &input_path, &output_path);
        assert_no_raw_runes(&reference_js);
        assert_js_parses_as_module(&reference_js);

        let allocator = Allocator::default();
        let parsed = parse(case.source, &allocator, false);
        assert!(
            parsed.errors.is_empty(),
            "parse failed for `{}` with {} errors",
            case.name,
            parsed.errors.len()
        );
        let analysis = analyze(&parsed.root);
        let actual_js = transform(&parsed.root, &analysis).js;

        assert_no_raw_runes(&actual_js);
        assert_js_parses_as_module(&actual_js);

        let expected_declarations = normalize_relevant_declaration_lines(&reference_js);
        let actual_declarations = normalize_relevant_declaration_lines(&actual_js);
        assert_eq!(
            actual_declarations, expected_declarations,
            "rune lowering declaration mismatch for `{}`\nreference decls: {:?}\nactual decls: {:?}\nreference js:\n{}\nactual js:\n{}",
            case.name, expected_declarations, actual_declarations, reference_js, actual_js
        );
    }
}

fn assert_no_raw_runes(js: &str) {
    for forbidden in [
        "$state",
        "$derived",
        "$props.id",
        "$props(",
        "$bindable",
        "$effect",
        "$inspect",
        "$host",
    ] {
        assert!(
            !js.contains(forbidden),
            "expected rune token `{forbidden}` to be lowered, got js:\n{js}"
        );
    }
}

fn assert_js_parses_as_module(js: &str) {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, js, SourceType::mjs()).parse();
    assert!(
        parsed.errors.is_empty(),
        "generated js failed to parse: {:?}\nsource:\n{}",
        parsed.errors,
        js
    );
}

fn normalize_relevant_declaration_lines(js: &str) -> Vec<String> {
    js.lines()
        .map(str::trim)
        .filter(|line| line.starts_with("let ") || line.starts_with("const "))
        .filter(|line| !line.starts_with("let __lux_") && !line.starts_with("const __lux_"))
        .map(canonicalize_declaration_line)
        .collect()
}

fn canonicalize_declaration_line(line: &str) -> String {
    line.replace("__lux_props_id()", "$PROPS_ID()")
        .replace("$.props_id($$renderer)", "$PROPS_ID()")
        .replace("$$props", "$PROPS")
        .replace("_props", "$PROPS")
        .replace("void 0", "undefined")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn run_reference_compile(runner_dir: &Path, input_path: &Path, output_path: &Path) -> String {
    let script_path = runner_dir.join("compile_server.mjs");
    let run = Command::new(node_executable())
        .arg(&script_path)
        .arg(input_path)
        .arg(output_path)
        .current_dir(runner_dir)
        .output()
        .unwrap_or_else(|error| panic!("failed to run node for {}: {error}", input_path.display()));

    assert!(
        run.status.success(),
        "reference compile failed for {}\nstdout:\n{}\nstderr:\n{}",
        input_path.display(),
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr),
    );

    let raw = fs::read_to_string(output_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", output_path.display()));
    let output: Value = serde_json::from_str(&raw)
        .unwrap_or_else(|error| panic!("invalid JSON {}: {error}", output_path.display()));

    output
        .get("js")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned()
}
