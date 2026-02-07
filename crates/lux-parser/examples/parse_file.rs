use oxc_allocator::Allocator;

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

    if !result.errors.is_empty() {
        eprintln!("=== Errors ({}) ===", result.errors.len());
        for err in &result.errors {
            eprintln!("  {:?}", err);
        }
        eprintln!();
    }

    println!("{:#?}", result.root);
}
