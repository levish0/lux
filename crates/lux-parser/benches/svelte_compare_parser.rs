use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use oxc_allocator::Allocator;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

struct BenchContext {
    source: String,
    input_path: PathBuf,
    runner_dir: PathBuf,
}

impl BenchContext {
    fn new() -> Self {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = manifest_dir
            .parent()
            .and_then(Path::parent)
            .expect("failed to resolve workspace root");

        let input_path = std::env::var("LUX_BENCH_INPUT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| workspace_root.join("ToParse.svelte"));
        let source = fs::read_to_string(&input_path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", input_path.display()));

        let runner_dir = workspace_root.join("crates/lux-parser/tests/tools/svelte_runner");
        ensure_svelte_runner(&runner_dir);

        Self {
            source,
            input_path,
            runner_dir,
        }
    }

    fn run_svelte_parse(&self, iterations: u64) -> Duration {
        let script_path = self.runner_dir.join("benchmark_phase.mjs");
        let output = Command::new(npx_executable())
            .arg("--yes")
            .arg("node")
            .arg(script_path)
            .arg("parse")
            .arg(&self.input_path)
            .arg(iterations.to_string())
            .current_dir(&self.runner_dir)
            .output()
            .unwrap_or_else(|err| panic!("failed to run npx parse benchmark: {err}"));

        assert!(
            output.status.success(),
            "svelte parse benchmark failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        let elapsed_ns = stdout
            .split_whitespace()
            .next()
            .unwrap_or("0")
            .parse::<u64>()
            .unwrap_or_else(|err| panic!("invalid elapsed ns from svelte runner: {err}"));

        Duration::from_nanos(elapsed_ns)
    }
}

fn npm_executable() -> &'static str {
    if cfg!(windows) { "npm.cmd" } else { "npm" }
}

fn npx_executable() -> &'static str {
    if cfg!(windows) { "npx.cmd" } else { "npx" }
}

fn ensure_svelte_runner(runner_dir: &Path) {
    let script_path = runner_dir.join("benchmark_phase.mjs");
    assert!(script_path.exists(), "missing {}", script_path.display());

    let svelte_module = runner_dir.join("node_modules/svelte/package.json");
    if svelte_module.exists() {
        return;
    }

    let install = Command::new(npm_executable())
        .arg("install")
        .arg("--silent")
        .arg("--no-fund")
        .arg("--no-audit")
        .current_dir(runner_dir)
        .output()
        .unwrap_or_else(|err| {
            panic!(
                "failed to run npm install in {}: {err}",
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
}

fn bench_parser(c: &mut Criterion) {
    let ctx = BenchContext::new();
    let mut group = c.benchmark_group("parser");
    group.throughput(Throughput::Bytes(ctx.source.len() as u64));

    group.bench_function("lux_parse", |b| {
        b.iter(|| {
            let allocator = Allocator::default();
            let result = lux_parser::parse(&ctx.source, &allocator, true);
            criterion::black_box(result.root.fragment.nodes.len());
        });
    });

    group.bench_function("svelte_parse_npx", |b| {
        b.iter_custom(|iters| ctx.run_svelte_parse(iters));
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(8));
    targets = bench_parser
}
criterion_main!(benches);
