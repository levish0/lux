# lux

Svelte compiler implemented in Rust. Aims for 100% compatibility with the official Svelte compiler.
## Architecture

Pipeline: **Parse → Analyze → Transform**

- **svelte-ast**: AST type definitions, ESTree serialization
- **svelte-parser**: Zero-copy parsing with OXC allocator, winnow-based template/CSS parser, OXC for JS/TS expressions
- **svelte-analyzer**: Semantic analysis (scope, bindings, validation)
- **svelte-transformer**: Code generation (JS/CSS output)

## Benchmarking

Compare Lux parser/analyzer pipeline against Svelte parser/compiler-analysis:

```bash
cargo bench -p lux-parser --bench svelte_compare_parser
cargo bench -p lux-analyzer --bench svelte_compare
```

The benchmark uses `npx node` in `crates/lux-parser/tests/tools/svelte_runner` for Svelte runs.
Set `LUX_BENCH_INPUT` to benchmark a different `.svelte` file.

## License

MIT
