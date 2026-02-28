use oxc_ast::ast::Expression;
use winnow::Result;
use winnow::combinator::opt;
use winnow::prelude::*;
use winnow::token::literal;

use crate::input::Input;
use crate::parser::read::expression::read_expression_until;
use crate::parser::utils::helpers::skip_whitespace;

pub(super) fn parse_snippet_params<'a>(input: &mut Input<'a>) -> Result<Vec<Expression<'a>>> {
    let mut params = Vec::new();

    loop {
        skip_whitespace(input);

        let remaining: &str = &input.input;
        if remaining.starts_with(')') {
            break;
        }

        let expr = read_expression_until(input, b",)")?;
        params.push(expr);

        skip_whitespace(input);

        // Comma between params.
        if opt(literal(",")).parse_next(input)?.is_none() {
            break;
        }
    }

    Ok(params)
}
