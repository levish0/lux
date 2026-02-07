use oxc_allocator::Allocator;
use std::io::Write;

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "ToParse.svelte".to_string());

    let source = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("Failed to read {path}: {e}");
        std::process::exit(1);
    });

    let allocator = Allocator::default();
    let result = lux_parser::parse(&source, &allocator, true);

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

    writeln!(out, "{:#?}", result.root).unwrap();

    println!("Output written to {}", output_path.display());
}
