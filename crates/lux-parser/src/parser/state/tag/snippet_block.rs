use lux_ast::common::Span;
use lux_ast::template::block::SnippetBlock;
use lux_ast::template::root::FragmentNode;
use oxc_ast::ast::Expression;
use winnow::Result;
use winnow::combinator::opt;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::{literal, take_while};

use crate::input::Input;
use crate::parser::read::expression::read_expression_until;
use crate::parser::state::fragment::parse_block_fragment;
use crate::parser::utils::helpers::{eat_block_close, require_whitespace, skip_whitespace};

/// Parse `{#snippet name(params)}...{/snippet}`.
/// Assumes `{` and `#` already consumed.
pub fn parse_snippet_block<'a>(input: &mut Input<'a>, start: usize) -> Result<FragmentNode<'a>> {
    literal("snippet").parse_next(input)?;
    require_whitespace(input)?;

    // Read snippet name as identifier
    let name_start = input.current_token_start();
    let name: &str = take_while(1.., |c: char| {
        c.is_ascii_alphanumeric() || c == '_' || c == '$'
    })
    .parse_next(input)?;
    let name_end = input.previous_token_end();

    let expression = oxc_ast::ast::IdentifierReference {
        span: oxc_span::Span::new(name_start as u32, name_end as u32),
        name: oxc_span::Atom::from(name).into(),
        reference_id: std::cell::Cell::new(None),
    };

    skip_whitespace(input);

    // Optional type parameters: <T, U>
    let type_params = if input.state.ts {
        let remaining: &str = &input.input;
        if remaining.starts_with('<') {
            read_type_params(input)?
        } else {
            None
        }
    } else {
        None
    };

    skip_whitespace(input);

    // Optional parameters: (a, b, c)
    let parameters = if opt(literal("(")).parse_next(input)?.is_some() {
        let params = parse_snippet_params(input)?;
        literal(")").parse_next(input)?;
        skip_whitespace(input);
        params
    } else {
        Vec::new()
    };

    literal("}").parse_next(input)?;

    let body = parse_block_fragment(input)?;

    eat_block_close(input, "snippet")?;
    let end = input.previous_token_end();

    Ok(FragmentNode::SnippetBlock(SnippetBlock {
        span: Span::new(start as u32, end as u32),
        expression,
        type_params,
        parameters,
        body,
    }))
}

fn read_type_params<'a>(input: &mut Input<'a>) -> Result<Option<&'a str>> {
    use crate::parser::utils::bracket::find_matching_bracket;

    let template = input.state.template;
    let pos = input.current_token_start();

    // Find matching '>'
    if let Some(end) = find_matching_bracket(template, pos + 1, '<') {
        let params = &template[pos..=end];
        // Advance input past the angle brackets
        let advance = end + 1 - pos;
        for _ in 0..advance {
            let _: char = winnow::token::any.parse_next(input)?;
        }
        Ok(Some(params))
    } else {
        Ok(None)
    }
}

fn parse_snippet_params<'a>(input: &mut Input<'a>) -> Result<Vec<Expression<'a>>> {
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

        // Comma between params
        if opt(literal(",")).parse_next(input)?.is_none() {
            break;
        }
    }

    Ok(params)
}
