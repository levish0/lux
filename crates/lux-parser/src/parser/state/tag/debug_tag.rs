use lux_ast::common::Span;
use lux_ast::template::root::FragmentNode;
use lux_ast::template::tag::DebugTag;
use oxc_ast::ast::{Expression, IdentifierReference};
use winnow::Result;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::literal;

use crate::input::Input;
use crate::parser::read::expression::read_expression;
use crate::parser::utils::helpers::skip_whitespace;

/// Parse `{@debug}` or `{@debug id1, id2, ...}`.
/// Assumes `{` already consumed. Starts at `@debug`.
pub fn parse_debug_tag<'a>(input: &mut Input<'a>, start: usize) -> Result<FragmentNode<'a>> {
    literal("@debug").parse_next(input)?;
    skip_whitespace(input);

    let remaining: &str = &input.input;
    let identifiers = if remaining.starts_with('}') {
        // {@debug} — debug all
        Vec::new()
    } else {
        // {@debug expr} — parse and extract identifiers
        let expression = read_expression(input)?;
        skip_whitespace(input);
        extract_identifiers(expression)
    };

    literal("}").parse_next(input)?;
    let end = input.previous_token_end();

    Ok(FragmentNode::DebugTag(DebugTag {
        span: Span::new(start as u32, end as u32),
        identifiers,
    }))
}

fn extract_identifiers<'a>(expr: Expression<'a>) -> Vec<IdentifierReference<'a>> {
    match expr {
        Expression::Identifier(id) => vec![id.unbox()],
        Expression::SequenceExpression(seq) => {
            let seq = seq.unbox();
            seq.expressions
                .into_iter()
                .filter_map(|e| {
                    if let Expression::Identifier(id) = e {
                        Some(id.unbox())
                    } else {
                        None
                    }
                })
                .collect()
        }
        _ => Vec::new(),
    }
}
