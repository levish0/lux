use oxc_ast::ast::Expression;
use winnow::Result;

use crate::input::Input;
use crate::parser::read::expression::read_expression;
use crate::parser::utils::helpers::{require_whitespace, skip_whitespace};

pub(super) fn parse_optional_clause_binding<'a>(
    input: &mut Input<'a>,
) -> Result<Option<Expression<'a>>> {
    let remaining: &str = &input.input;
    if !remaining.trim_start().starts_with('}') {
        require_whitespace(input)?;
        let expr = read_expression(input)?;
        skip_whitespace(input);
        Ok(Some(expr))
    } else {
        skip_whitespace(input);
        Ok(None)
    }
}
