use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

pub fn workspace_root_from_manifest_dir(manifest_dir: &Path) -> PathBuf {
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("failed to resolve workspace root")
        .to_path_buf()
}

pub fn reference_root(workspace_root: &Path) -> PathBuf {
    workspace_root.join("svelte_reference/svelte-svelte-5.50.0/packages/svelte/tests")
}

pub fn npm_executable() -> &'static str {
    if cfg!(windows) { "npm.cmd" } else { "npm" }
}

pub fn node_executable() -> &'static str {
    if cfg!(windows) { "node.exe" } else { "node" }
}

pub fn ensure_svelte_runner(workspace_root: &Path) -> PathBuf {
    let runner_dir = workspace_root.join("tools/svelte_runner");
    assert!(runner_dir.exists(), "missing {}", runner_dir.display());

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

pub fn run_node_script<I, S>(runner_dir: &Path, script_name: &str, args: I) -> Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let script_path = runner_dir.join(script_name);
    assert!(script_path.exists(), "missing {}", script_path.display());

    Command::new(node_executable())
        .arg(&script_path)
        .args(args)
        .current_dir(runner_dir)
        .output()
        .unwrap_or_else(|error| panic!("failed to run node for {}: {error}", script_path.display()))
}

pub fn read_json(path: &Path) -> Value {
    let raw = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    serde_json::from_str(&raw)
        .unwrap_or_else(|error| panic!("invalid JSON {}: {error}", path.display()))
}

pub fn normalize_text(input: &str) -> String {
    input.replace('\r', "").trim().to_string()
}

pub fn is_loose_parser_sample(sample_name: &str) -> bool {
    sample_name.starts_with("loose-")
}

pub fn is_legacy_reference_sample(sample_name: &str) -> bool {
    sample_name.contains("legacy")
}
