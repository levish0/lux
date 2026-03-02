use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use lux_analyzer::analyze;
use lux_parser::parse;
use lux_transformer::transform;
use oxc_allocator::Allocator;
use serde_json::Value;

#[derive(Clone, Copy)]
struct ParityCase<'a> {
    name: &'a str,
    source: &'a str,
}

#[test]
fn parity_reference_server_imports_smoke() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("failed to resolve workspace root");
    let runner_dir = ensure_svelte_runner(workspace_root);
    let generated_dir = manifest_dir.join("target/parity-reference-transform-imports");
    let _ = fs::create_dir_all(&generated_dir);

    let cases = vec![
        ParityCase {
            name: "instance_before_module",
            source: r#"
<script context="module">
  import { m } from './m';
</script>
<script>
  import x from './x';
</script>
<p>{x}</p>
"#,
        },
        ParityCase {
            name: "typescript_type_imports",
            source: r#"
<script lang="ts">
  import type { A } from './types';
  import { type B, c } from './mixed';
  import {} from './side-effect';
</script>
{c}
"#,
        },
        ParityCase {
            name: "module_reexports_and_type_exports",
            source: r#"
<script context="module" lang="ts">
  export type { T } from './types';
  export { value, type TypeOnly } from './mixed';
  export { default as def } from './def';
  export * from './all-values';
</script>
<p>ok</p>
"#,
        },
        ParityCase {
            name: "module_value_exports_with_typescript",
            source: r#"
<script context="module" lang="ts">
  export const answer: number = 42;
  export function greet(name: string): string {
    return name;
  }
</script>
<p>{answer}</p>
"#,
        },
    ];

    for case in cases {
        let input_path = generated_dir.join(format!("{}.svelte", case.name));
        let output_path = generated_dir.join(format!("{}.reference.json", case.name));
        fs::write(&input_path, case.source)
            .unwrap_or_else(|error| panic!("failed to write {}: {error}", input_path.display()));

        let reference_js = run_reference_compile(&runner_dir, &input_path, &output_path);
        let expected_module_lines = normalize_relevant_module_lines(&reference_js);
        let expected_export_decls = normalize_relevant_export_declaration_lines(&reference_js);

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
        let actual_module_lines = normalize_relevant_module_lines(&actual_js);
        let actual_export_decls = normalize_relevant_export_declaration_lines(&actual_js);

        assert_eq!(
            actual_module_lines, expected_module_lines,
            "module line parity mismatch for `{}`\nreference lines: {:?}\nactual lines: {:?}\nreference js:\n{}\nactual js:\n{}",
            case.name, expected_module_lines, actual_module_lines, reference_js, actual_js
        );
        assert_eq!(
            actual_export_decls, expected_export_decls,
            "export declaration parity mismatch for `{}`\nreference exports: {:?}\nactual exports: {:?}\nreference js:\n{}\nactual js:\n{}",
            case.name, expected_export_decls, actual_export_decls, reference_js, actual_js
        );
    }
}

fn normalize_relevant_module_lines(js: &str) -> Vec<String> {
    js.lines()
        .map(str::trim)
        .filter(|line| {
            line.starts_with("import ") || (line.starts_with("export ") && line.contains(" from "))
        })
        .filter(|line| !line.contains("svelte/internal/"))
        .filter(|line| !line.contains("lux/runtime/"))
        .map(canonicalize_module_line)
        .collect()
}

fn canonicalize_module_line(line: &str) -> String {
    line.replace('\'', "\"")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_relevant_export_declaration_lines(js: &str) -> Vec<String> {
    js.lines()
        .map(str::trim)
        .filter(|line| {
            line.starts_with("export const ")
                || line.starts_with("export let ")
                || line.starts_with("export var ")
                || line.starts_with("export function ")
                || line.starts_with("export class ")
        })
        .map(canonicalize_module_line)
        .collect()
}

fn npm_executable() -> &'static str {
    if cfg!(windows) { "npm.cmd" } else { "npm" }
}

fn node_executable() -> &'static str {
    if cfg!(windows) { "node.exe" } else { "node" }
}

fn ensure_svelte_runner(workspace_root: &Path) -> PathBuf {
    let runner_dir = workspace_root.join("tools/svelte_runner");
    let script_path = runner_dir.join("compile_server.mjs");
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
