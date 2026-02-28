use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use oxc_allocator::Allocator;

fn main() {
    let mut args = env::args();
    let _ = args.next();

    let input_path = args.next().unwrap_or_else(|| "ToParse.svelte".to_string());
    let output_path = args
        .next()
        .unwrap_or_else(|| "ToParse.transformed.txt".to_string());

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

    let mut lines = Vec::new();
    lines.push(format!("input={input_path}"));
    lines.push(format!("diagnostics={}", analysis.diagnostics.len()));
    lines.push(format!("js={:?}", transformed.js));
    lines.push(format!(
        "css_present={}",
        transformed.css.as_ref().is_some()
    ));
    lines.push(format!(
        "css_hash={}",
        transformed.css_hash.as_deref().unwrap_or("none")
    ));
    lines.push(format!(
        "css_scope={}",
        transformed.css_scope.as_deref().unwrap_or("none")
    ));

    if let Some(css) = transformed.css.as_deref() {
        lines.push(format!("css={css:?}"));
    }

    let output_path = PathBuf::from(output_path);
    fs::write(&output_path, lines.join("\n"))
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", output_path.display()));

    let parse_throughput =
        input_len as f64 / 1024.0 / parse_duration.as_secs_f64().max(f64::EPSILON);
    let analyze_throughput =
        input_len as f64 / 1024.0 / analyze_duration.as_secs_f64().max(f64::EPSILON);
    let transform_throughput =
        input_len as f64 / 1024.0 / transform_duration.as_secs_f64().max(f64::EPSILON);

    println!("Output written to {}", output_path.display());
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
