use oxc_allocator::Allocator;
use std::io::Write;
use std::time::Instant;

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "ToParse.svelte".to_string());

    let source = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("Failed to read {path}: {e}");
        std::process::exit(1);
    });
    let input_len = source.len();

    let start_time = Instant::now();
    let allocator = Allocator::default();
    let result = lux_parser::parse(&source, &allocator, true);
    let duration = start_time.elapsed();

    let output_path = std::path::Path::new(&path).with_extension("parsed.txt");

    let mut out = std::fs::File::create(&output_path).unwrap_or_else(|e| {
        eprintln!("Failed to create {}: {e}", output_path.display());
        std::process::exit(1);
    });

    if !result.errors.is_empty() {
        writeln!(out, "=== Errors ({}) ===", result.errors.len()).unwrap();
        for err in &result.errors {
            writeln!(out, "  {:?}", err).unwrap();
        }
        writeln!(out).unwrap();
    }

    if !result.warnings.is_empty() {
        writeln!(out, "=== Warnings ({}) ===", result.warnings.len()).unwrap();
        for warning in &result.warnings {
            writeln!(out, "  {:?}", warning).unwrap();
        }
        writeln!(out).unwrap();
    }

    writeln!(out, "{:#?}", result.root).unwrap();

    println!("Output written to {}", output_path.display());
    println!(
        "Parsed {} top-level nodes in {:?}",
        result.root.fragment.nodes.len(),
        duration
    );
    println!(
        "Performance: {:.2} KB/s",
        input_len as f64 / 1024.0 / duration.as_secs_f64().max(f64::EPSILON)
    );
}
