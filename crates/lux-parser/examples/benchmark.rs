use oxc_allocator::Allocator;
use std::hint::black_box;
use std::time::Instant;

fn benchmark_parse(content: &str, iterations: u32) -> (f64, usize, usize, usize) {
    let input_bytes = content.len();

    // Warmup
    for _ in 0..3 {
        let allocator = Allocator::default();
        let result = lux_parser::parse(content, &allocator, true);
        black_box(result.root.fragment.nodes.len());
    }

    // Benchmark
    let start_time = Instant::now();
    let mut top_level_nodes = 0usize;
    let mut error_count = 0usize;
    let mut warning_count = 0usize;

    for _ in 0..iterations {
        let allocator = Allocator::default();
        let result = lux_parser::parse(content, &allocator, true);
        top_level_nodes = result.root.fragment.nodes.len();
        error_count = result.errors.len();
        warning_count = result.warnings.len();

        black_box(top_level_nodes);
        black_box(error_count);
        black_box(warning_count);
    }

    let duration = start_time.elapsed();
    let avg_duration_ms = duration.as_secs_f64() * 1000.0 / iterations as f64;
    let throughput_mb_s =
        (input_bytes as f64 * iterations as f64) / (1024.0 * 1024.0) / duration.as_secs_f64();

    println!("Input: {} bytes", input_bytes);
    println!(
        "Top-level nodes: {}, errors: {}, warnings: {}",
        top_level_nodes, error_count, warning_count
    );
    println!("Avg parse time: {:.3} ms", avg_duration_ms);
    println!(
        "Total time for {} iterations: {:.3} s",
        iterations,
        duration.as_secs_f64()
    );
    println!("Performance: {:.2} MB/s", throughput_mb_s);
    println!();

    (throughput_mb_s, top_level_nodes, error_count, warning_count)
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "ToParse.svelte".to_string());
    let iterations = std::env::args()
        .nth(2)
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(100);

    if iterations == 0 {
        eprintln!("Iterations must be at least 1");
        std::process::exit(1);
    }

    let input_content = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("Failed to read {path}: {e}");
        std::process::exit(1);
    });

    println!("{}", "=".repeat(60));
    println!("Lux Parser Benchmark");
    println!("File: {path}");
    println!("{}", "=".repeat(60));

    let _ = benchmark_parse(&input_content, iterations);

    println!("{}", "=".repeat(60));
    println!();
    println!("Testing with 10x content size...");

    let larger_content = input_content.repeat(10);
    let larger_iterations = (iterations / 10).max(1);
    let _ = benchmark_parse(&larger_content, larger_iterations);
}
