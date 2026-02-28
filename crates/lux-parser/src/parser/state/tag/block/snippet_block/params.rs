use oxc_ast::ast::BindingPattern;
use winnow::Result;
use winnow::combinator::opt;
use winnow::prelude::*;
use winnow::token::literal;

use crate::input::Input;
use crate::parser::read::pattern::read_binding_pattern_until;
use crate::parser::utils::helpers::skip_whitespace;

pub(super) fn parse_snippet_params<'a>(input: &mut Input<'a>) -> Result<Vec<BindingPattern<'a>>> {
    let mut params = Vec::new();

    loop {
        skip_whitespace(input);

        let remaining: &str = &input.input;
        if remaining.starts_with(')') {
            break;
        }

        let expr = read_binding_pattern_until(input, b",)")?;
        params.push(expr);

        skip_whitespace(input);

        // Comma between params.
        if opt(literal(",")).parse_next(input)?.is_none() {
            break;
        }
    }

    Ok(params)
}
