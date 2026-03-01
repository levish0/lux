use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use oxc_allocator::Allocator;

fn main() {
    let mut args = env::args();
    let _ = args.next();

    let input_path = args.next().unwrap_or_else(|| "ToParse.svelte".to_string());
    let output_js_path = args
        .next()
        .unwrap_or_else(|| "ToParse.transformed.js".to_string());
    let output_css_path = args.next();

    let source = fs::read_to_string(&input_path)
        .unwrap_or_else(|err| panic!("failed to read `{input_path}`: {err}"));
    let input_len = source.len();

    let parse_start = Instant::now();
    let allocator = Allocator::default();
    let parsed = lux_parser::parse(&source, &allocator, true);
    let parse_duration = parse_start.elapsed();
    if !parsed.errors.is_empty() {
        panic!(
            "parse failed with {} error(s); first: {}",
            parsed.errors.len(),
            parsed.errors[0].message
        );
    }

    let analyze_start = Instant::now();
    let analysis = lux_analyzer::analyze(&parsed.root);
    let analyze_duration = analyze_start.elapsed();

    let transform_start = Instant::now();
    let transformed = lux_transformer::transform(&parsed.root, &analysis);
    let transform_duration = transform_start.elapsed();

    let output_js_path = PathBuf::from(output_js_path);
    fs::write(&output_js_path, &transformed.js)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", output_js_path.display()));
    let runtime_written_paths = write_runtime_modules(
        output_js_path.parent().unwrap_or_else(|| Path::new(".")),
        &transformed.runtime_modules,
    );

    let css_written_path = if let Some(css) = transformed.css.as_deref() {
        let css_path = output_css_path
            .map(PathBuf::from)
            .unwrap_or_else(|| output_js_path.with_extension("css"));
        fs::write(&css_path, css)
            .unwrap_or_else(|err| panic!("failed to write {}: {err}", css_path.display()));
        Some(css_path)
    } else {
        None
    };

    println!("Input: {input_path}");
    println!("Diagnostics: {}", analysis.diagnostics.len());
    println!("JS written: {}", output_js_path.display());
    if runtime_written_paths.is_empty() {
        println!("Runtime modules written: none");
    } else {
        println!("Runtime modules written:");
        for path in &runtime_written_paths {
            println!("  - {}", path.display());
        }
    }
    if let Some(css_path) = css_written_path {
        println!("CSS written: {}", css_path.display());
    } else {
        println!("CSS written: none");
    }
    println!(
        "CSS hash: {}",
        transformed.css_hash.as_deref().unwrap_or("none")
    );
    println!(
        "CSS scope: {}",
        transformed.css_scope.as_deref().unwrap_or("none")
    );

    let parse_throughput =
        input_len as f64 / 1024.0 / parse_duration.as_secs_f64().max(f64::EPSILON);
    let analyze_throughput =
        input_len as f64 / 1024.0 / analyze_duration.as_secs_f64().max(f64::EPSILON);
    let transform_throughput =
        input_len as f64 / 1024.0 / transform_duration.as_secs_f64().max(f64::EPSILON);

    println!(
        "Parse: {:?} ({:.2} KB/s), Analyze: {:?} ({:.2} KB/s), Transform: {:?} ({:.2} KB/s)",
        parse_duration,
        parse_throughput,
        analyze_duration,
        analyze_throughput,
        transform_duration,
        transform_throughput
    );
}

fn write_runtime_modules(base_dir: &Path, modules: &[lux_transformer::RuntimeModule]) -> Vec<PathBuf> {
    let mut paths = Vec::with_capacity(modules.len());

    for module in modules {
        let mut output_path = base_dir.to_path_buf();
        for segment in module.specifier.split('/') {
            output_path.push(segment);
        }
        if output_path.extension().is_none() {
            output_path.set_extension("js");
        }

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).unwrap_or_else(|err| {
                panic!("failed to create runtime module dir {}: {err}", parent.display())
            });
        }
        fs::write(&output_path, &module.code).unwrap_or_else(|err| {
            panic!("failed to write runtime module {}: {err}", output_path.display())
        });
        paths.push(output_path);
    }

    paths
}
