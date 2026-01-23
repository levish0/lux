# svelte-rs

Svelte 5 compiler implemented in Rust. Aims to produce output identical to the official Svelte compiler.

## Architecture

Pipeline: **Parse → Analyze → Transform**

- **svelte-ast**: AST type definitions, ESTree serialization
- **svelte-parser**: winnow-based template/CSS parser, SWC for JS/TS expressions
- **svelte-analyzer**: Semantic analysis (scope, bindings, validation)
- **svelte-transformer**: Code generation (JS/CSS output)

## License

MIT
