use std::fs;
use std::process;
use std::time::Instant;

use oxc_allocator::Allocator;
use lux_parser::{ParseOptions, parse};

fn main() {
    let path = "ToParse.svelte";
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to read {}: {}", path, e);
            process::exit(1);
        }
    };

    let document_len = source.len();
    println!("Input ({} bytes):\n{}\n", document_len, "=".repeat(50));

    let allocator = Allocator::default();
    let start_time = Instant::now();
    let root = match parse(&source, &allocator, ParseOptions::default()) {
        Ok(root) => root,
        Err(errors) => {
            eprintln!("Parse errors:");
            for err in &errors {
                eprintln!("  {:?}", err);
            }
            process::exit(1);
        }
    };
    let duration = start_time.elapsed();

    println!("Parsed in {:?}", duration);
    println!("Fragment nodes: {}", root.fragment.nodes.len());
    println!("Has instance script: {}", root.instance.is_some());
    println!("Has module script: {}", root.module.is_some());
    println!("Has CSS: {}", root.css.is_some());
    println!(
        "Performance: {:.2} KB/s",
        document_len as f64 / 1024.0 / duration.as_secs_f64()
    );

    // Write JSON output
    let json = serde_json::to_string_pretty(&root).expect("Failed to serialize");
    fs::write("ToParse.json", &json).expect("Failed to write ToParse.json");
    println!("Written ToParse.json ({} bytes)", json.len());
}
