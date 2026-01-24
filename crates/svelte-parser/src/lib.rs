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

#[cfg(test)]
mod tests {
    #[test]
    fn debug_oxc_formal_parameter() {
        use oxc_allocator::Allocator;
        use oxc_span::SourceType;
        use oxc_estree::CompactTSSerializer;
        use oxc_estree::ESTree;
        use oxc_ast::ast::{Expression, Statement};

        let allocator = Allocator::default();
        let source = "(n: number) => {}";
        let source_type = SourceType::ts();

        let result = oxc_parser::Parser::new(&allocator, source, source_type).parse();
        assert!(result.errors.is_empty(), "parse errors: {:?}", result.errors);

        let program = result.program;
        let stmt = &program.body[0];
        if let Statement::ExpressionStatement(expr_stmt) = stmt {
            if let Expression::ArrowFunctionExpression(arrow) = &expr_stmt.expression {
                let params = &arrow.params;
                let param = &params.items[0];
                eprintln!("param.type_annotation is_some: {}", param.type_annotation.is_some());

                // Serialize FormalParameter
                let mut ser = CompactTSSerializer::default();
                param.serialize(&mut ser);
                let param_json = ser.into_string();
                eprintln!("FormalParameter JSON: {}", param_json);

                // Serialize BindingPattern
                let mut ser2 = CompactTSSerializer::default();
                param.pattern.serialize(&mut ser2);
                let pattern_json = ser2.into_string();
                eprintln!("BindingPattern JSON: {}", pattern_json);
            }
        }
    }
}
