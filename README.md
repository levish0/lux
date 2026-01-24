# svelte-rs

Svelte 5+ compiler implemented in Rust. Aims for 100% compatibility with the official Svelte compiler.
## Architecture

Pipeline: **Parse → Analyze → Transform**

- **svelte-ast**: AST type definitions, ESTree serialization
- **svelte-parser**: winnow-based template/CSS parser, OXC for JS/TS expressions
- **svelte-analyzer**: Semantic analysis (scope, bindings, validation)
- **svelte-transformer**: Code generation (JS/CSS output)

## License

MIT
