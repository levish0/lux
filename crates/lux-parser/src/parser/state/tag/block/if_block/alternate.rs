use lux_ast::common::Span;
use lux_ast::template::block::IfBlock;
use lux_ast::template::root::{Fragment, FragmentNode};
use winnow::Result;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::literal;

use crate::input::Input;
use crate::parser::read::expression::read_expression;
use crate::parser::state::fragment::parse_block_fragment;
use crate::parser::utils::helpers::{
    at_block_continuation, eat_block_continuation, require_whitespace, skip_whitespace,
};

pub(super) fn parse_if_alternate<'a>(input: &mut Input<'a>) -> Result<Option<Fragment<'a>>> {
    if !at_block_continuation(input, "else") {
        return Ok(None);
    }

    let elseif_start = input.current_token_start();
    eat_block_continuation(input, "else")?;
    skip_whitespace(input);

    let remaining: &str = &input.input;
    if remaining.starts_with("if") {
        parse_else_if(input, elseif_start)
    } else {
        parse_else(input)
    }
}

fn parse_else_if<'a>(input: &mut Input<'a>, elseif_start: usize) -> Result<Option<Fragment<'a>>> {
    literal("if").parse_next(input)?;
    require_whitespace(input)?;

    let test = read_expression(input)?;
    skip_whitespace(input);
    literal("}").parse_next(input)?;

    let consequent = parse_block_fragment(input)?;
    let alternate = parse_if_alternate(input)?;

    let end = input.current_token_start();

    let if_block = FragmentNode::IfBlock(IfBlock {
        span: Span::new(elseif_start as u32, end as u32),
        elseif: true,
        test,
        consequent,
        alternate,
    });

    Ok(Some(Fragment {
        nodes: vec![if_block],
        transparent: true,
        dynamic: false,
    }))
}

fn parse_else<'a>(input: &mut Input<'a>) -> Result<Option<Fragment<'a>>> {
    skip_whitespace(input);
    literal("}").parse_next(input)?;
    let body = parse_block_fragment(input)?;
    Ok(Some(body))
}
