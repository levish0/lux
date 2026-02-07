use lux_ast::common::Span;
use lux_ast::template::root::FragmentNode;
use lux_ast::template::tag::{ConstDeclaration, ConstTag};
use winnow::Result;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::literal;

use crate::input::Input;
use crate::parser::read::expression::{read_expression, read_expression_until};
use crate::parser::utils::helpers::{require_whitespace, skip_whitespace};

/// Parse `{@const id = expression}`.
/// Assumes `{` already consumed. Starts at `@const`.
pub fn parse_const_tag<'a>(input: &mut Input<'a>, start: usize) -> Result<FragmentNode<'a>> {
    literal("@const").parse_next(input)?;
    require_whitespace(input)?;

    let decl_start = input.current_token_start();

    // Read id (left side of =), stopping at `=`
    let id = read_expression_until(input, b"=")?;
    skip_whitespace(input);

    literal("=").parse_next(input)?;
    skip_whitespace(input);

    let init = read_expression(input)?;
    skip_whitespace(input);
    literal("}").parse_next(input)?;

    let end = input.previous_token_end();
    let decl_end = end - 1; // before closing }

    Ok(FragmentNode::ConstTag(ConstTag {
        span: Span::new(start as u32, end as u32),
        declaration: ConstDeclaration {
            span: Span::new(decl_start as u32, decl_end as u32),
            id,
            init,
        },
        metadata: None,
    }))
}
