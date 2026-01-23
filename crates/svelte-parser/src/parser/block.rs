use svelte_ast::blocks::{AwaitBlock, EachBlock, IfBlock, KeyBlock, SnippetBlock};
use svelte_ast::node::FragmentNode;
use svelte_ast::root::Fragment;
use svelte_ast::span::Span;
use swc_ecma_ast as swc;
use winnow::Result as ParseResult;
use winnow::combinator::{opt, peek, repeat_till};
use winnow::prelude::*;
use winnow::stream::Location;
use winnow::token::{literal, take, take_while};

use super::ParserInput;
use super::bracket::{
    read_until_chars_balanced, read_until_close_brace, read_until_keyword_balanced,
};
use super::fragment::fragment_node_parser;
use super::swc_parse::{parse_expression, parse_param_list, parse_pattern};

/// Dispatch `{#block}` to the appropriate block parser.
pub fn block_parser(parser_input: &mut ParserInput) -> ParseResult<FragmentNode> {
    let start = parser_input.current_token_start();

    // consume {#
    literal("{#").parse_next(parser_input)?;

    // read block keyword
    let keyword: &str =
        take_while(1.., |c: char| c.is_ascii_alphabetic()).parse_next(parser_input)?;

    match keyword {
        "if" => if_block_parser(parser_input, start),
        "each" => each_block_parser(parser_input, start),
        "await" => await_block_parser(parser_input, start),
        "key" => key_block_parser(parser_input, start),
        "snippet" => snippet_block_parser(parser_input, start),
        _ => Err(winnow::error::ContextError::new()),
    }
}

/// Parse `{#if test}...{:else if test}...{:else}...{/if}`
fn if_block_parser(parser_input: &mut ParserInput, start: usize) -> ParseResult<FragmentNode> {
    take_while(1.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;

    // read test expression up to }
    let offset = parser_input.current_token_start();
    let content = read_until_close_brace(parser_input)?;
    let test = parse_expression(content, parser_input.state.ts, offset as u32)?;
    literal("}").parse_next(parser_input)?;

    // parse consequent body until {:else}, {:else if}, or {/if}
    let (consequent_nodes, _): (Vec<FragmentNode>, _) = repeat_till(
        0..,
        fragment_node_parser,
        peek(block_continuation_or_close("if")),
    )
    .parse_next(parser_input)?;

    // check for alternate
    let alternate = parse_if_alternate(parser_input)?;

    // consume {/if}
    if alternate.is_none() {
        block_close("if").parse_next(parser_input)?;
    }

    let end = parser_input.previous_token_end();

    Ok(FragmentNode::IfBlock(IfBlock {
        span: Span::new(start, end),
        elseif: false,
        test,
        consequent: Fragment {
            nodes: consequent_nodes,
        },
        alternate,
    }))
}

fn parse_if_alternate(parser_input: &mut ParserInput) -> ParseResult<Option<Fragment>> {
    // Try to consume {:else - if not present, no alternate
    if opt(literal("{:else")).parse_next(parser_input)?.is_none() {
        return Ok(None);
    }

    // Check for {:else if ...} vs {:else}
    let ws: &str = take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;

    if !ws.is_empty() && opt(literal("if")).parse_next(parser_input)?.is_some() {
        // {:else if test}...
        take_while(1.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;

        let nested_start = parser_input.current_token_start();
        // Adjust start back to {:else if position
        let nested_start = nested_start - "{:else if ".len();

        let offset = parser_input.current_token_start();
        let content = read_until_close_brace(parser_input)?;
        let test = parse_expression(content, parser_input.state.ts, offset as u32)?;
        literal("}").parse_next(parser_input)?;

        let (consequent_nodes, _): (Vec<FragmentNode>, _) = repeat_till(
            0..,
            fragment_node_parser,
            peek(block_continuation_or_close("if")),
        )
        .parse_next(parser_input)?;

        let alternate = parse_if_alternate(parser_input)?;

        if alternate.is_none() {
            block_close("if").parse_next(parser_input)?;
        }

        let end = parser_input.previous_token_end();

        let nested_if = FragmentNode::IfBlock(IfBlock {
            span: Span::new(nested_start, end),
            elseif: true,
            test,
            consequent: Fragment {
                nodes: consequent_nodes,
            },
            alternate,
        });

        Ok(Some(Fragment {
            nodes: vec![nested_if],
        }))
    } else {
        // {:else}
        literal("}").parse_next(parser_input)?;

        let (nodes, _): (Vec<FragmentNode>, _) =
            repeat_till(0.., fragment_node_parser, peek(block_close_peek("if")))
                .parse_next(parser_input)?;

        block_close("if").parse_next(parser_input)?;

        Ok(Some(Fragment { nodes }))
    }
}

/// Parse `{#each expr as item, index (key)}...{:else}...{/each}`
fn each_block_parser(parser_input: &mut ParserInput, start: usize) -> ParseResult<FragmentNode> {
    take_while(1.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;

    // Read expression until " as " keyword
    let expr_offset = parser_input.current_token_start();
    let expr_text = read_until_keyword_balanced(parser_input, " as ")?;
    let expression = parse_expression(expr_text, parser_input.state.ts, expr_offset as u32)?;

    literal(" as ").parse_next(parser_input)?;
    take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;

    // Read context pattern until , or ( or } at depth 0
    let ctx_offset = parser_input.current_token_start();
    let context_text = read_until_chars_balanced(parser_input, &[',', '(', '}'])?;
    let context = if context_text.trim().is_empty() {
        None
    } else {
        Some(parse_pattern(
            context_text,
            parser_input.state.ts,
            ctx_offset as u32,
        )?)
    };

    // Optional index
    let index = if opt(literal(",")).parse_next(parser_input)?.is_some() {
        take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;
        let idx: &str = take_while(1.., |c: char| c.is_ascii_alphanumeric() || c == '_')
            .parse_next(parser_input)?;
        take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;
        Some(idx.to_string())
    } else {
        None
    };

    // Optional key
    let key = if opt(literal("(")).parse_next(parser_input)?.is_some() {
        let key_offset = parser_input.current_token_start();
        let key_content = read_until_close_paren(parser_input)?;
        let key_expr = parse_expression(key_content, parser_input.state.ts, key_offset as u32)?;
        literal(")").parse_next(parser_input)?;
        take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;
        Some(key_expr)
    } else {
        None
    };

    literal("}").parse_next(parser_input)?;

    // Parse body
    let (body_nodes, _): (Vec<FragmentNode>, _) = repeat_till(
        0..,
        fragment_node_parser,
        peek(block_continuation_or_close("each")),
    )
    .parse_next(parser_input)?;

    // Optional {:else} fallback
    let fallback = if opt(literal("{:else}")).parse_next(parser_input)?.is_some() {
        let (nodes, _): (Vec<FragmentNode>, _) =
            repeat_till(0.., fragment_node_parser, peek(block_close_peek("each")))
                .parse_next(parser_input)?;
        Some(Fragment { nodes })
    } else {
        None
    };

    block_close("each").parse_next(parser_input)?;
    let end = parser_input.previous_token_end();

    Ok(FragmentNode::EachBlock(EachBlock {
        span: Span::new(start, end),
        expression,
        context,
        body: Fragment { nodes: body_nodes },
        fallback,
        index,
        key,
    }))
}

/// Parse `{#key expr}...{/key}`
fn key_block_parser(parser_input: &mut ParserInput, start: usize) -> ParseResult<FragmentNode> {
    take_while(1.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;

    let offset = parser_input.current_token_start();
    let content = read_until_close_brace(parser_input)?;
    let expression = parse_expression(content, parser_input.state.ts, offset as u32)?;
    literal("}").parse_next(parser_input)?;

    let (nodes, _): (Vec<FragmentNode>, _) =
        repeat_till(0.., fragment_node_parser, block_close("key")).parse_next(parser_input)?;

    let end = parser_input.previous_token_end();

    Ok(FragmentNode::KeyBlock(KeyBlock {
        span: Span::new(start, end),
        expression,
        fragment: Fragment { nodes },
    }))
}

/// Parse `{#await expr}...{:then value}...{:catch error}...{/await}`
fn await_block_parser(parser_input: &mut ParserInput, start: usize) -> ParseResult<FragmentNode> {
    take_while(1.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;

    let offset = parser_input.current_token_start();
    let content = read_until_close_brace(parser_input)?;
    let expression = parse_expression(content, parser_input.state.ts, offset as u32)?;
    literal("}").parse_next(parser_input)?;

    // Parse pending body
    let (pending_nodes, _): (Vec<FragmentNode>, _) = repeat_till(
        0..,
        fragment_node_parser,
        peek(block_continuation_or_close("await")),
    )
    .parse_next(parser_input)?;

    let pending = if pending_nodes.is_empty() {
        None
    } else {
        Some(Fragment {
            nodes: pending_nodes,
        })
    };

    let mut then_fragment: Option<Fragment> = None;
    let mut catch_fragment: Option<Fragment> = None;
    let mut value: Option<Box<swc::Pat>> = None;
    let mut error: Option<Box<swc::Pat>> = None;

    // {:then value}
    if opt(literal("{:then")).parse_next(parser_input)?.is_some() {
        take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;

        if opt(peek(literal("}"))).parse_next(parser_input)?.is_none() {
            let pat_offset = parser_input.current_token_start();
            let pat_content = read_until_close_brace(parser_input)?;
            value = Some(parse_pattern(
                pat_content,
                parser_input.state.ts,
                pat_offset as u32,
            )?);
        }
        literal("}").parse_next(parser_input)?;

        let (nodes, _): (Vec<FragmentNode>, _) = repeat_till(
            0..,
            fragment_node_parser,
            peek(block_continuation_or_close("await")),
        )
        .parse_next(parser_input)?;
        then_fragment = Some(Fragment { nodes });
    }

    // {:catch error}
    if opt(literal("{:catch")).parse_next(parser_input)?.is_some() {
        take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;

        if opt(peek(literal("}"))).parse_next(parser_input)?.is_none() {
            let pat_offset = parser_input.current_token_start();
            let pat_content = read_until_close_brace(parser_input)?;
            error = Some(parse_pattern(
                pat_content,
                parser_input.state.ts,
                pat_offset as u32,
            )?);
        }
        literal("}").parse_next(parser_input)?;

        let (nodes, _): (Vec<FragmentNode>, _) =
            repeat_till(0.., fragment_node_parser, peek(block_close_peek("await")))
                .parse_next(parser_input)?;
        catch_fragment = Some(Fragment { nodes });
    }

    block_close("await").parse_next(parser_input)?;
    let end = parser_input.previous_token_end();

    Ok(FragmentNode::AwaitBlock(AwaitBlock {
        span: Span::new(start, end),
        expression,
        value,
        error,
        pending,
        then: then_fragment,
        catch: catch_fragment,
    }))
}

/// Parse `{#snippet name(params)}...{/snippet}`
fn snippet_block_parser(parser_input: &mut ParserInput, start: usize) -> ParseResult<FragmentNode> {
    take_while(1.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;

    // Read snippet name
    let name: &str = take_while(1.., |c: char| c.is_ascii_alphanumeric() || c == '_')
        .parse_next(parser_input)?;
    let ident = Box::new(swc::Ident::new(
        name.into(),
        swc_common::DUMMY_SP,
        Default::default(),
    ));

    // Parse parameters
    literal("(").parse_next(parser_input)?;
    let params_content = read_until_close_paren(parser_input)?;
    literal(")").parse_next(parser_input)?;
    take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;
    literal("}").parse_next(parser_input)?;

    let parameters = parse_param_list(params_content, parser_input.state.ts)?;

    let (nodes, _): (Vec<FragmentNode>, _) =
        repeat_till(0.., fragment_node_parser, block_close("snippet")).parse_next(parser_input)?;

    let end = parser_input.previous_token_end();

    Ok(FragmentNode::SnippetBlock(SnippetBlock {
        span: Span::new(start, end),
        expression: ident,
        parameters,
        type_params: None,
        body: Fragment { nodes },
    }))
}

// --- Helper parsers ---

/// Read until `)` at bracket depth 0, without consuming the `)`.
fn read_until_close_paren<'i>(input: &mut ParserInput<'i>) -> ParseResult<&'i str> {
    use super::bracket::find_matching_bracket_for_close_char;
    let remaining: &str = &input.input;
    let end = find_matching_bracket_for_close_char(remaining, ')')
        .ok_or(winnow::error::ContextError::new())?;
    take(end).parse_next(input)
}

/// Parser that matches `{:...}` or `{/name}`
fn block_continuation_or_close<'a>(
    name: &'a str,
) -> impl FnMut(&mut ParserInput<'_>) -> ParseResult<()> + 'a {
    move |input: &mut ParserInput| {
        let check: ParseResult<&str> = peek(literal("{:")).parse_next(input);
        if check.is_ok() {
            return Ok(());
        }
        peek(block_close_peek(name)).parse_next(input)
    }
}

/// Peek for `{/name}` without consuming.
fn block_close_peek<'a>(name: &'a str) -> impl FnMut(&mut ParserInput<'_>) -> ParseResult<()> + 'a {
    move |input: &mut ParserInput| {
        literal("{/").parse_next(input)?;
        literal(name).parse_next(input)?;
        literal("}").parse_next(input)?;
        Ok(())
    }
}

/// Consume `{/name}`.
fn block_close<'a>(name: &'a str) -> impl FnMut(&mut ParserInput<'_>) -> ParseResult<()> + 'a {
    move |input: &mut ParserInput| {
        take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(input)?;
        literal("{/").parse_next(input)?;
        literal(name).parse_next(input)?;
        literal("}").parse_next(input)?;
        Ok(())
    }
}
