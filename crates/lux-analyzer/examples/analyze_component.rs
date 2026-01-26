//! Example: Run analyze_component on a Svelte component from ToParse.svelte
//!
//! Usage:
//!   cargo run --example analyze_component
//!
//! This example reads ToParse.svelte from the project root and runs semantic analysis on it.

use std::fs;
use std::process;
use std::time::Instant;

use lux_analyzer::{analyze_component, AnalyzeOptions};
use lux_parser::{parse, ParseOptions};
use oxc_allocator::Allocator;

fn main() {
    // Use command line argument if provided, otherwise default to ToParse.svelte
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).map(String::as_str).unwrap_or("ToParse.svelte");
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to read {}: {}", path, e);
            process::exit(1);
        }
    };

    let document_len = source.len();
    println!("Input ({} bytes):\n{}\n", document_len, "=".repeat(50));

    // Parse the Svelte component
    let allocator = Allocator::default();
    let parse_start = Instant::now();
    let mut root = match parse(&source, &allocator, ParseOptions::default()) {
        Ok(root) => root,
        Err(errors) => {
            eprintln!("Parse errors:");
            for err in &errors {
                eprintln!("  {:?}", err);
            }
            process::exit(1);
        }
    };
    let parse_duration = parse_start.elapsed();
    println!("Parsed in {:?}", parse_duration);

    // Analyze the component
    let analyze_start = Instant::now();
    let analyze_options = AnalyzeOptions {
        filename: path.to_string(),
        runes: None, // Auto-detect
        ..Default::default()
    };
    let analysis = analyze_component(&source, &mut root, analyze_options);
    let analyze_duration = analyze_start.elapsed();
    println!("Analyzed in {:?}", analyze_duration);
    println!();

    // Print analysis results
    println!("=== Component Analysis ===");
    println!("Name: {}", analysis.base.name);
    println!("Runes mode: {}", analysis.runes);
    println!("Immutable: {}", analysis.base.immutable);
    println!("Accessors: {}", analysis.base.accessors);
    println!("CSS hash: {}", analysis.css.hash);
    println!();

    // Print exports
    println!("=== Exports ({}) ===", analysis.exports.len());
    for export in &analysis.exports {
        println!("  {} - alias: {:?}", export.name, export.alias);
    }
    println!();

    // Print declarations in root scope
    let root_scope = analysis.scope_tree.get_scope(analysis.scope_tree.root_scope_id());
    println!("=== Root Scope Declarations ({}) ===", root_scope.declarations.len());
    for (name, binding_id) in root_scope.declarations.iter() {
        let binding = analysis.scope_tree.get_binding(*binding_id);
        println!("  {} - kind: {:?}, reassigned: {}", name, binding.kind, binding.reassigned);
    }
    println!();

    // Print references in root scope
    println!("=== Root Scope References ({}) ===", root_scope.references.len());
    for (name, refs) in root_scope.references.iter() {
        println!("  {} - {} references", name, refs.len());
    }
    println!();

    // Print errors and warnings
    if !analysis.errors.is_empty() {
        println!("=== Errors ({}) ===", analysis.errors.len());
        for error in &analysis.errors {
            println!("  {:?}", error);
        }
        println!();
    }

    if !analysis.warnings.is_empty() {
        println!("=== Warnings ({}) ===", analysis.warnings.len());
        for warning in &analysis.warnings {
            println!("  {:?}", warning);
        }
        println!();
    }

    println!(
        "Performance: {:.2} KB/s (parse + analyze)",
        document_len as f64 / 1024.0 / (parse_duration + analyze_duration).as_secs_f64()
    );
}
