use oxc_ast::ast::Expression;
use winnow::Result;
use winnow::combinator::opt;
use winnow::error::ContextError;
use winnow::prelude::*;
use winnow::token::take;
use winnow::token::{literal, take_while};

use crate::input::Input;
use crate::parser::read::expression::read_expression_until;
use crate::parser::utils::scanner::scan_expression_boundary;
use crate::parser::utils::helpers::{require_whitespace, skip_whitespace};

pub(super) fn parse_each_context<'a>(input: &mut Input<'a>) -> Result<Option<Expression<'a>>> {
    if opt(literal("as")).parse_next(input)?.is_some() {
        require_whitespace(input)?;
        match read_expression_until(input, b",(") {
            Ok(expr) => Ok(Some(expr)),
            Err(_) => {
                // Pattern context (e.g. `{ a = 1 }`) is valid in Svelte `each`,
                // but not always a valid JS expression. Consume it so parsing can continue.
                let remaining: &str = &input.input;
                let end = scan_expression_boundary(remaining, b",(").ok_or_else(ContextError::new)?;
                let pattern_source = remaining[..end].trim_end();
                if pattern_source.is_empty() {
                    return Err(ContextError::new());
                }
                let _ = take(pattern_source.len()).parse_next(input)?;
                Ok(None)
            }
        }
    } else {
        Ok(None)
    }
}

pub(super) fn parse_each_index<'a>(input: &mut Input<'a>) -> Result<Option<&'a str>> {
    skip_whitespace(input);

    if opt(literal(",")).parse_next(input)?.is_some() {
        skip_whitespace(input);
        let name: &str = take_while(1.., |c: char| {
            c.is_ascii_alphanumeric() || c == '_' || c == '$'
        })
        .parse_next(input)?;
        skip_whitespace(input);
        Ok(Some(name))
    } else {
        Ok(None)
    }
}

pub(super) fn parse_each_key<'a>(input: &mut Input<'a>) -> Result<Option<Expression<'a>>> {
    skip_whitespace(input);

    if opt(literal("(")).parse_next(input)?.is_some() {
        skip_whitespace(input);
        let key_expr = read_expression_until(input, b")")?;
        skip_whitespace(input);
        literal(")").parse_next(input)?;
        skip_whitespace(input);
        Ok(Some(key_expr))
    } else {
        Ok(None)
    }
}
