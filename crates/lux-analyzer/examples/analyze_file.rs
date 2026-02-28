use std::io::Write;
use std::time::Instant;

use oxc_allocator::Allocator;

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "ToParse.svelte".to_string());

    let source = std::fs::read_to_string(&path).unwrap_or_else(|error| {
        eprintln!("Failed to read {path}: {error}");
        std::process::exit(1);
    });
    let input_len = source.len();

    let allocator = Allocator::default();

    let parse_start = Instant::now();
    let parsed = lux_parser::parse(&source, &allocator, true);
    let parse_duration = parse_start.elapsed();

    let analyze_start = Instant::now();
    let tables = lux_analyzer::analyze(&parsed.root);
    let analyze_duration = analyze_start.elapsed();

    let output_path = std::path::Path::new(&path).with_extension("analyzed.txt");
    let mut output = std::fs::File::create(&output_path).unwrap_or_else(|error| {
        eprintln!("Failed to create {}: {error}", output_path.display());
        std::process::exit(1);
    });

    if !parsed.errors.is_empty() {
        writeln!(output, "=== Parse Errors ({}) ===", parsed.errors.len()).unwrap();
        for error in &parsed.errors {
            writeln!(output, "  {:?}", error).unwrap();
        }
        writeln!(output).unwrap();
    }

    if !parsed.warnings.is_empty() {
        writeln!(output, "=== Parse Warnings ({}) ===", parsed.warnings.len()).unwrap();
        for warning in &parsed.warnings {
            writeln!(output, "  {:?}", warning).unwrap();
        }
        writeln!(output).unwrap();
    }

    writeln!(output, "=== Analyzer Summary ===").unwrap();
    writeln!(output, "script_scopes: {}", tables.script_scopes.len()).unwrap();
    writeln!(output, "script_symbols: {}", tables.script_symbols.len()).unwrap();
    writeln!(output, "script_references: {}", tables.script_references.len()).unwrap();
    writeln!(output, "template_scopes: {}", tables.template_scopes.len()).unwrap();
    writeln!(output, "template_bindings: {}", tables.template_bindings.len()).unwrap();
    writeln!(
        output,
        "template_references: {}",
        tables.template_references.len()
    )
    .unwrap();
    writeln!(output, "css_rules: {}", tables.css_rules.len()).unwrap();
    writeln!(output, "diagnostics: {}", tables.diagnostics.len()).unwrap();
    writeln!(output).unwrap();

    if !tables.diagnostics.is_empty() {
        writeln!(output, "=== Diagnostics ===").unwrap();
        for diagnostic in &tables.diagnostics {
            writeln!(
                output,
                "  {:?} {:?} [{}..{}] {}",
                diagnostic.severity,
                diagnostic.code,
                diagnostic.span.start,
                diagnostic.span.end,
                diagnostic.message
            )
            .unwrap();
        }
        writeln!(output).unwrap();
    }

    writeln!(output, "=== Analysis Tables (Debug) ===").unwrap();
    writeln!(output, "{:#?}", tables).unwrap();

    let parse_throughput =
        input_len as f64 / 1024.0 / parse_duration.as_secs_f64().max(f64::EPSILON);
    let analyze_throughput =
        input_len as f64 / 1024.0 / analyze_duration.as_secs_f64().max(f64::EPSILON);

    println!("Output written to {}", output_path.display());
    println!(
        "Parse: {:?} ({:.2} KB/s), Analyze: {:?} ({:.2} KB/s)",
        parse_duration, parse_throughput, analyze_duration, analyze_throughput
    );
}
