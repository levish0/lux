use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;
use svelte_ast::utils::estree::{clear_loc_source, set_loc_source};
use svelte_parser::{ParseOptions, parse};

fn tests_dir() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests"))
}

fn reference_tests_dir() -> PathBuf {
    tests_dir().join("svelte@5.48.0-tests")
}

fn output_dir() -> PathBuf {
    tests_dir().join("output")
}

fn run_cmd(program: &str, args: &[&str], dir: &Path) {
    let status = if cfg!(windows) {
        let cmd_args = std::iter::once(program)
            .chain(args.iter().copied())
            .collect::<Vec<_>>()
            .join(" ");
        Command::new("cmd")
            .args(["/C", &cmd_args])
            .current_dir(dir)
            .status()
    } else {
        Command::new(program).args(args).current_dir(dir).status()
    };
    let status = status.unwrap_or_else(|e| panic!("Failed to run {}: {}", program, e));
    assert!(status.success(), "{} failed with {:?}", program, status);
}

fn generate_svelte_ast(tests_dir: &Path) {
    let node_modules = tests_dir.join("node_modules");
    if !node_modules.exists() {
        run_cmd("pnpm", &["install"], tests_dir);
    }
    run_cmd("node", &["generate.mjs"], tests_dir);
}

/// Check if `actual` is a superset of `expected`.
/// All fields in `expected` must exist in `actual` with matching values.
/// Extra fields in `actual` are allowed (EXTRA is not a mismatch).
fn is_superset(actual: &Value, expected: &Value) -> bool {
    match (actual, expected) {
        (Value::Object(a), Value::Object(e)) => {
            for (key, e_val) in e {
                match a.get(key) {
                    Some(a_val) => {
                        if !is_superset(a_val, e_val) {
                            return false;
                        }
                    }
                    None => return false, // missing field
                }
            }
            true
        }
        (Value::Array(a), Value::Array(e)) => {
            if a.len() != e.len() {
                return false;
            }
            a.iter()
                .zip(e.iter())
                .all(|(a_item, e_item)| is_superset(a_item, e_item))
        }
        _ => actual == expected,
    }
}

const SKIP_CATEGORIES: &[&str] = &["parser-legacy"];

/// Recursively find all .svelte files
fn find_svelte_files(dir: &Path, results: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            find_svelte_files(&path, &mut *results);
        } else if path.extension().is_some_and(|e| e == "svelte") {
            results.push(path);
        }
    }
}

/// Build a unique name from relative path, matching generate.mjs logic
fn build_name(file: &Path, base: &Path) -> String {
    let rel = file.strip_prefix(base).unwrap();
    let parts: Vec<&str> = rel
        .iter()
        .filter_map(|p| p.to_str())
        .filter(|p| *p != "samples")
        .collect();
    let mut name_parts: Vec<String> = parts.iter().map(|p| p.to_string()).collect();
    // Remove .svelte extension from last part
    if let Some(last) = name_parts.last_mut() {
        *last = last.trim_end_matches(".svelte").to_string();
    }
    name_parts.join("--")
}

#[test]
fn generate_and_compare() {
    let tests = tests_dir();
    let ref_tests = reference_tests_dir();

    if !ref_tests.exists() {
        eprintln!("Skipping: svelte@5.48.0-tests/ not found.");
        return;
    }

    let mut files = Vec::new();
    find_svelte_files(&ref_tests, &mut files);
    files.retain(|f| {
        let rel = f.strip_prefix(&ref_tests).unwrap();
        let first = rel.iter().next().and_then(|p| p.to_str()).unwrap_or("");
        !SKIP_CATEGORIES.contains(&first)
    });
    files.sort();

    if files.is_empty() {
        eprintln!("No .svelte files found.");
        return;
    }

    // Generate svelte.json via node
    generate_svelte_ast(&tests);

    let out_base = output_dir();
    fs::create_dir_all(&out_base).unwrap();

    let mut mismatches = Vec::new();
    let mut parse_errors = Vec::new();
    let mut ok_count = 0usize;
    let mut skip_count = 0usize;

    for file in &files {
        let name = build_name(file, &ref_tests);

        let source = fs::read_to_string(file)
            .unwrap()
            .replace("\r\n", "\n")
            .trim_end()
            .to_string();

        let options = if name.contains("loose-") {
            ParseOptions { loose: true }
        } else {
            ParseOptions::default()
        };

        let out_dir = out_base.join(&name);
        fs::create_dir_all(&out_dir).unwrap();

        // Skip if svelte itself couldn't parse it
        if out_dir.join("svelte-error.txt").exists() {
            skip_count += 1;
            continue;
        }

        match parse(&source, options) {
            Ok(root) => {
                set_loc_source(&source);
                let mut actual: Value = serde_json::to_value(&root).unwrap();
                clear_loc_source();

                if let Value::Object(ref mut obj) = actual {
                    obj.remove("comments");
                }

                let lux_json = serde_json::to_string_pretty(&actual).unwrap();
                fs::write(out_dir.join("lux.json"), &lux_json).unwrap();

                let svelte_path = out_dir.join("svelte.json");
                if svelte_path.exists() {
                    let svelte_str = fs::read_to_string(&svelte_path).unwrap();
                    let svelte_val: Value = serde_json::from_str(&svelte_str).unwrap();
                    if is_superset(&actual, &svelte_val) {
                        ok_count += 1;
                    } else {
                        mismatches.push(name.clone());
                    }
                }
            }
            Err(errs) => {
                let err_msg = format!("{:?}", errs);
                fs::write(out_dir.join("lux-error.txt"), &err_msg).unwrap();
                parse_errors.push(name.clone());
            }
        }
    }

    println!("\n=== Results ===");
    println!("Total: {}", files.len());
    println!("OK (match): {}", ok_count);
    println!("Mismatch: {}", mismatches.len());
    println!("Parse errors: {}", parse_errors.len());
    println!("Skipped (svelte error): {}", skip_count);

    if !mismatches.is_empty() || !parse_errors.is_empty() {
        if !parse_errors.is_empty() {
            println!("\n--- Parse Errors ({}) ---", parse_errors.len());
            for name in &parse_errors {
                println!("  {}", name);
            }
        }
        if !mismatches.is_empty() {
            println!("\n--- Mismatches ({}) ---", mismatches.len());
            for name in &mismatches {
                println!("  {}", name);
            }
        }
        panic!(
            "{} mismatches, {} parse errors",
            mismatches.len(),
            parse_errors.len()
        );
    }
}
