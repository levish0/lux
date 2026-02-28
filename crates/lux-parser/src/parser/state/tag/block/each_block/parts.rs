use oxc_ast::ast::{BindingPattern, Expression};
use winnow::Result;
use winnow::combinator::opt;
use winnow::prelude::*;
use winnow::token::{literal, take_while};

use crate::input::Input;
use crate::parser::read::expression::read_expression_until;
use crate::parser::read::pattern::read_binding_pattern_until;
use crate::parser::utils::helpers::{require_whitespace, skip_whitespace};

pub(super) fn parse_each_context<'a>(input: &mut Input<'a>) -> Result<Option<BindingPattern<'a>>> {
    if opt(literal("as")).parse_next(input)?.is_some() {
        require_whitespace(input)?;
        let pattern = read_binding_pattern_until(input, b",(")?;
        Ok(Some(pattern))
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
