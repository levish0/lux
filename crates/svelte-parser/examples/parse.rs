use std::fs;
use std::process;
use std::time::Instant;

use svelte_parser::{ParseOptions, parse};

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

    let start_time = Instant::now();
    let root = match parse(&source, ParseOptions::default()) {
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

    let json = serde_json::to_string_pretty(&root).unwrap();
    fs::write("ToParse.json", &json).unwrap();

    println!("\nResult saved to ToParse.json");
    println!(
        "Performance: {:.2} KB/s",
        document_len as f64 / 1024.0 / duration.as_secs_f64()
    );
}
