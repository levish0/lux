# lux

A Blazing-Fast Compiler for Svelte 5.
## Architecture

Pipeline: **Parse → Analyze → Transform**

- **svelte-ast**: AST type definitions, ESTree serialization
- **svelte-parser**: Zero-copy parsing with OXC allocator, winnow-based template/CSS parser, OXC for JS/TS expressions
- **svelte-analyzer**: Semantic analysis (scope, bindings, validation)
- **svelte-transformer**: Code generation (JS/CSS output)

## Benchmarking

Compare Lux parser/analyzer/transform pipeline against Svelte:

```bash
cargo bench -p lux-parser --bench svelte_compare_parser
cargo bench -p lux-analyzer --bench svelte_compare_analyzer
cargo bench -p lux-transformer --bench svelte_compare_transformer
```

The benchmark uses `node` in `tools/svelte_runner` for Svelte runs.
Default input is `benchmarks/assets/benchmark.svelte`.
Set `LUX_BENCH_INPUT` to benchmark a different `.svelte` file.
Reports are written under:
- `benchmarks/criterion/lux-parser`
- `benchmarks/criterion/lux-analyzer`
- `benchmarks/criterion/lux-transformer`

## License

MIT
