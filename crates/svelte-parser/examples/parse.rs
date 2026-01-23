use std::fs;
use std::process;

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

    let json = serde_json::to_string_pretty(&root).unwrap();
    println!("{}", json);
    fs::write("ToParse.json", &json).unwrap();
    eprintln!("Written to ToParse.json");
}
