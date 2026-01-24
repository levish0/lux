pub mod error;
mod parser;

use oxc_allocator::Allocator;
use svelte_ast::root::Root;

use crate::error::ParseError;

#[derive(Debug, Clone, Default)]
pub struct ParseOptions {
    pub loose: bool,
}

/// Parse a Svelte template into an AST.
pub fn parse<'a>(
    source: &'a str,
    allocator: &'a Allocator,
    options: ParseOptions,
) -> Result<Root<'a>, Vec<ParseError>> {
    let parser = parser::Parser::new(source, allocator, options.loose);

    if parser.errors.is_empty() {
        Ok(parser.into_root())
    } else {
        let errors = parser
            .errors
            .iter()
            .map(|e| {
                ParseError::new(
                    e.kind,
                    svelte_ast::span::Span::new(e.position, e.position),
                    &e.message,
                )
            })
            .collect();
        Err(errors)
    }
}
