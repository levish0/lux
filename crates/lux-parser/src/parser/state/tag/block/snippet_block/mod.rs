use lux_ast::common::Span;
use lux_ast::template::block::SnippetBlock;
use lux_ast::template::root::FragmentNode;
use winnow::Result;
use winnow::combinator::opt;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::{literal, take_while};

use crate::input::Input;
use crate::parser::state::fragment::parse_block_fragment;
use crate::parser::utils::helpers::{eat_block_close, require_whitespace, skip_whitespace};

mod params;
mod type_params;

use params::parse_snippet_params;
use type_params::read_type_params;

/// Parse `{#snippet name(params)}...{/snippet}`.
/// Assumes `{` and `#` are already consumed.
pub fn parse_snippet_block<'a>(input: &mut Input<'a>, start: usize) -> Result<FragmentNode<'a>> {
    literal("snippet").parse_next(input)?;
    require_whitespace(input)?;

    let expression = parse_snippet_name(input)?;
    skip_whitespace(input);

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

fn parse_snippet_name<'a>(input: &mut Input<'a>) -> Result<oxc_ast::ast::IdentifierReference<'a>> {
    let name_start = input.current_token_start();
    let name: &str = take_while(1.., |c: char| {
        c.is_ascii_alphanumeric() || c == '_' || c == '$'
    })
    .parse_next(input)?;
    let name_end = input.previous_token_end();

    Ok(oxc_ast::ast::IdentifierReference {
        node_id: std::cell::Cell::new(oxc_syntax::node::NodeId::DUMMY),
        span: oxc_span::Span::new(name_start as u32, name_end as u32),
        name: oxc_span::Atom::from(name).into(),
        reference_id: std::cell::Cell::new(None),
    })
}
