use std::cell::Cell;

use oxc_ast::ast::{BindingIdentifier, BindingPattern};
use oxc_span::SourceType;

use crate::error::ErrorKind;
use crate::parser::Parser;
use crate::parser::bracket::match_bracket;
use crate::parser::span_offset::shift_binding_pattern_spans;

/// Read a destructuring pattern at the current parser position.
/// Port of reference `read/context.js`.
///
/// 1. Tries `read_identifier()` first (simple variable name).
/// 2. If not, checks for `{` or `[` and uses `match_bracket` to find extent.
/// 3. Wraps as `let <pattern> = 1;` and parses with OXC to get BindingPattern.
pub fn read_pattern<'a>(parser: &mut Parser<'a>) -> Option<BindingPattern<'a>> {
    let start = parser.index;

    // 1. Try identifier first (matching reference: `const id = parser.read_identifier()`)
    let (name, id_start, id_end) = parser.read_identifier();
    if !name.is_empty() {
        // Simple identifier pattern — construct BindingPattern directly
        // TODO: read_type_annotation after identifier (for TS)
        return Some(BindingPattern::BindingIdentifier(
            oxc_allocator::Box::new_in(
                BindingIdentifier {
                    span: oxc_span::Span::new(id_start as u32, id_end as u32),
                    name: oxc_span::Atom::from(name),
                    symbol_id: Cell::new(None),
                },
                parser.allocator,
            ),
        ));
    }

    // 2. Check for destructuring pattern
    let ch = parser.template.as_bytes().get(parser.index).copied();
    if ch != Some(b'{') && ch != Some(b'[') {
        if !parser.loose {
            parser.error(
                ErrorKind::ExpectedExpression,
                start,
                "Expected pattern".to_string(),
            );
        }
        return None;
    }

    // 3. Use match_bracket to find the end of the pattern
    let default_brackets: &[(char, char)] = &[('{', '}'), ('(', ')'), ('[', ']')];
    let bracket_end = match match_bracket(parser.template, start, default_brackets) {
        Some(end) => end,
        None => {
            if !parser.loose {
                parser.error(
                    ErrorKind::ExpectedToken,
                    start,
                    "Unterminated pattern".to_string(),
                );
            }
            return None;
        }
    };

    parser.index = bracket_end;
    let pattern_string = &parser.template[start..bracket_end];

    // 4. Parse with OXC using `let <pattern> = 1;` (no padding needed).
    let let_prefix_len = 4u32; // "let "
    let snippet = format!("let {} = 1;", pattern_string);
    let snippet_str = parser.allocator.alloc_str(&snippet);

    let source_type = if parser.ts {
        SourceType::ts()
    } else {
        SourceType::mjs()
    };

    let result = oxc_parser::Parser::new(parser.allocator, snippet_str, source_type).parse();

    if !result.errors.is_empty() {
        if !parser.loose {
            let first_err = &result.errors[0];
            parser.error(
                ErrorKind::JsParseError,
                start,
                format!("Pattern parse error: {}", first_err),
            );
        }
        return None;
    }

    // 5. Extract BindingPattern and shift spans to match original positions.
    let program = result.program;
    let mut pattern = extract_pattern(program)?;

    // Pattern starts at byte 4 ("let ") in the snippet, but at `start` in the original.
    let offset = start as u32 - let_prefix_len;
    shift_binding_pattern_spans(&mut pattern, offset);

    Some(pattern)
}

/// Extract the BindingPattern from a parsed `let <pattern> = 1;`
fn extract_pattern(program: oxc_ast::ast::Program) -> Option<BindingPattern> {
    let body = program.body;
    if body.len() != 1 {
        return None;
    }

    let stmt = body.into_iter().next()?;
    match stmt {
        oxc_ast::ast::Statement::VariableDeclaration(decl) => {
            let decl = decl.unbox();
            let declarations = decl.declarations;
            if declarations.len() != 1 {
                return None;
            }
            let declarator = declarations.into_iter().next()?;
            Some(declarator.id)
        }
        _ => None,
    }
}
