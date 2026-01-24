//! Performance comparison: Rust parser vs Node.js Svelte parser
//!
//! Run with: cargo test --release -p svelte-parser --test performance -- --nocapture

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use oxc_allocator::Allocator;
use svelte_parser::{ParseOptions, parse};

fn tests_dir() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests"))
}

fn fixtures_dir() -> PathBuf {
    tests_dir().join("fixtures")
}

/// Run Node.js Svelte parser and return duration
fn run_node_parser(file_path: &std::path::Path, iterations: u32) -> Option<Duration> {
    let tests = tests_dir();
    
    // Ensure node_modules exists
    let node_modules = tests.join("node_modules");
    if !node_modules.exists() {
        let status = if cfg!(windows) {
            Command::new("cmd")
                .args(["/C", "pnpm install"])
                .current_dir(&tests)
                .status()
        } else {
            Command::new("pnpm")
                .args(["install"])
                .current_dir(&tests)
                .status()
        };
        if status.is_err() || !status.unwrap().success() {
            eprintln!("Failed to install node_modules");
            return None;
        }
    }
    
    // Create script that reads the file and measures parse time
    let file_path_str = file_path.to_string_lossy().replace("\\", "/");
    let script = format!(
        r#"
        const fs = require('fs');
        const {{ parse }} = require('svelte/compiler');
        const source = fs.readFileSync('{}', 'utf8');
        const iterations = {};
        
        // Warmup
        for (let i = 0; i < 3; i++) {{
            parse(source);
        }}
        
        const start = performance.now();
        for (let i = 0; i < iterations; i++) {{
            parse(source);
        }}
        const end = performance.now();
        
        console.log(JSON.stringify({{ elapsed_ms: end - start }}));
        "#,
        file_path_str,
        iterations,
    );
    
    let output = Command::new("node")
        .args(["-e", &script])
        .current_dir(&tests)
        .output();
    
    match output {
        Ok(out) => {
            if !out.status.success() {
                eprintln!("Node.js error: {}", String::from_utf8_lossy(&out.stderr));
                return None;
            }
            let stdout = String::from_utf8_lossy(&out.stdout);
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                let elapsed_ms = json["elapsed_ms"].as_f64()?;
                Some(Duration::from_secs_f64(elapsed_ms / 1000.0))
            } else {
                None
            }
        }
        Err(e) => {
            eprintln!("Failed to run Node.js: {}", e);
            None
        }
    }
}

/// Run Rust parser and return duration
fn run_rust_parser(source: &str, iterations: u32) -> Duration {
    // Warmup with fresh allocators
    for _ in 0..3 {
        let allocator = Allocator::default();
        let _ = parse(source, &allocator, ParseOptions::default());
    }
    
    let start = Instant::now();
    for _ in 0..iterations {
        // Note: allocator must be created each time because Root borrows from it
        let allocator = Allocator::default();
        let _ = parse(source, &allocator, ParseOptions::default());
    }
    start.elapsed()
}

fn format_duration(d: Duration) -> String {
    let ms = d.as_secs_f64() * 1000.0;
    if ms < 1.0 {
        format!("{:.3}ms", ms)
    } else if ms < 1000.0 {
        format!("{:.2}ms", ms)
    } else {
        format!("{:.2}s", ms / 1000.0)
    }
}

#[test]
fn performance_comparison() {
    let fixture = fixtures_dir().join("performance.svelte");
    
    if !fixture.exists() {
        eprintln!("Skipping: fixtures/performance.svelte not found.");
        return;
    }
    
    let source = fs::read_to_string(&fixture)
        .unwrap()
        .replace("\r\n", "\n");
    
    let source_lines = source.lines().count();
    let source_bytes = source.len();
    
    println!();
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Svelte Parser Performance Comparison");
    println!("═══════════════════════════════════════════════════════════════");
    println!("  File: performance.svelte");
    println!("  Lines: {}  |  Bytes: {}", source_lines, source_bytes);
    println!("═══════════════════════════════════════════════════════════════");
    println!();
    
    // Single parse comparison
    println!("── Single Parse ──");
    
    let rust_single = run_rust_parser(&source, 1);
    println!("  Rust:    {}", format_duration(rust_single));
    
    if let Some(node_single) = run_node_parser(&fixture, 1) {
        println!("  Node.js: {}", format_duration(node_single));
        
        let speedup = node_single.as_secs_f64() / rust_single.as_secs_f64();
        if speedup > 1.0 {
            println!("  → Rust is {:.3}x faster\n", speedup);
        } else {
            println!("  → Node.js is {:.3}x faster\n", 1.0 / speedup);
        }
    } else {
        println!("  Node.js: (skipped - not available)\n");
    }
    
    // Bulk parse comparison (100 iterations)
    let iterations = 100;
    println!("── {} Iterations ──", iterations);
    
    let rust_bulk = run_rust_parser(&source, iterations);
    let rust_per_parse = rust_bulk.as_secs_f64() * 1000.0 / iterations as f64;
    println!("  Rust:    {} total ({:.6}ms per parse)", format_duration(rust_bulk), rust_per_parse);
    
    if let Some(node_bulk) = run_node_parser(&fixture, iterations) {
        let node_per_parse = node_bulk.as_secs_f64() * 1000.0 / iterations as f64;
        println!("  Node.js: {} total ({:.6}ms per parse)", format_duration(node_bulk), node_per_parse);
        
        let speedup = node_bulk.as_secs_f64() / rust_bulk.as_secs_f64();
        if speedup > 1.0 {
            println!("  → Rust is {:.3}x faster\n", speedup);
        } else {
            println!("  → Node.js is {:.3}x faster\n", 1.0 / speedup);
        }
    } else {
        println!("  Node.js: (skipped - not available)\n");
    }
    
    // Throughput
    println!("── Throughput ──");
    let rust_throughput_mb = (source_bytes as f64 * iterations as f64) / rust_bulk.as_secs_f64() / 1_000_000.0;
    println!("  Rust:    {:.2} MB/s", rust_throughput_mb);
    
    println!("\n✓ Performance test completed");
}
