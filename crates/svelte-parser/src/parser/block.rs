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
use super::expression::{make_empty_ident, parse_expression_or_loose};
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
    let loose = parser_input.state.loose;
    let test = parse_expression_or_loose(content, parser_input.state.ts, offset as u32, loose)?;
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
        let loose = parser_input.state.loose;
        let test = parse_expression_or_loose(content, parser_input.state.ts, offset as u32, loose)?;
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

    let loose = parser_input.state.loose;

    // Read expression until " as " keyword
    let expr_offset = parser_input.current_token_start();
    let (expression, has_as) = match read_until_keyword_balanced(parser_input, " as ") {
        Ok(expr_text) => {
            let expr = parse_expression_or_loose(expr_text, parser_input.state.ts, expr_offset as u32, loose)?;
            (expr, true)
        }
        Err(_) if loose => {
            // No " as " found - read until } and use as expression
            let content = read_until_close_brace(parser_input)?;
            let expr = make_empty_ident(content, expr_offset as u32);
            (expr, false)
        }
        Err(e) => return Err(e),
    };

    let (context, index, key) = if has_as {
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
            let key_expr = parse_expression_or_loose(key_content, parser_input.state.ts, key_offset as u32, loose)?;
            literal(")").parse_next(parser_input)?;
            take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;
            Some(key_expr)
        } else {
            None
        };

        (context, index, key)
    } else {
        (None, None, None)
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
/// Also handles inline syntax: `{#await expr then value}` and `{#await expr catch error}`
fn await_block_parser(parser_input: &mut ParserInput, start: usize) -> ParseResult<FragmentNode> {
    take_while(1.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;

    let offset = parser_input.current_token_start();
    let loose = parser_input.state.loose;
    let ts = parser_input.state.ts;
    let content = read_until_close_brace(parser_input)?;
    literal("}").parse_next(parser_input)?;

    // Check for inline "then" or "catch" keywords in the expression content
    use super::bracket::find_keyword_at_depth_zero;
    let inline_then_pos = find_keyword_at_depth_zero(content, " then ");
    let inline_catch_pos = find_keyword_at_depth_zero(content, " catch ");

    let expression;
    let mut pending: Option<Fragment> = None;
    let mut then_fragment: Option<Fragment> = None;
    let mut catch_fragment: Option<Fragment> = None;
    let mut value: Option<Box<swc::Pat>> = None;
    let mut error: Option<Box<swc::Pat>> = None;

    if let Some(then_pos) = inline_then_pos {
        // Inline then: {#await expr then value}
        let expr_text = &content[..then_pos];
        expression = parse_expression_or_loose(expr_text, ts, offset as u32, loose)?;

        let value_text = &content[then_pos + " then ".len()..];
        if !value_text.trim().is_empty() {
            let val_offset = offset + then_pos + " then ".len();
            value = Some(parse_pattern(value_text, ts, val_offset as u32)?);
        }

        // Parse then body until {/await}
        let (nodes, _): (Vec<FragmentNode>, _) =
            repeat_till(0.., fragment_node_parser, peek(block_close_peek("await")))
                .parse_next(parser_input)?;
        then_fragment = Some(Fragment { nodes });
    } else if let Some(catch_pos) = inline_catch_pos {
        // Inline catch: {#await expr catch error}
        let expr_text = &content[..catch_pos];
        expression = parse_expression_or_loose(expr_text, ts, offset as u32, loose)?;

        let error_text = &content[catch_pos + " catch ".len()..];
        if !error_text.trim().is_empty() {
            let err_offset = offset + catch_pos + " catch ".len();
            error = Some(parse_pattern(error_text, ts, err_offset as u32)?);
        }

        // Parse catch body until {/await}
        let (nodes, _): (Vec<FragmentNode>, _) =
            repeat_till(0.., fragment_node_parser, peek(block_close_peek("await")))
                .parse_next(parser_input)?;
        catch_fragment = Some(Fragment { nodes });
    } else {
        // Normal form: {#await expr}...{:then}...{:catch}...{/await}
        expression = parse_expression_or_loose(content, ts, offset as u32, loose)?;

        // Parse pending body
        let (pending_nodes, _): (Vec<FragmentNode>, _) = repeat_till(
            0..,
            fragment_node_parser,
            peek(block_continuation_or_close("await")),
        )
        .parse_next(parser_input)?;
        pending = Some(Fragment { nodes: pending_nodes });

        // {:then value}
        if opt(literal("{:then")).parse_next(parser_input)?.is_some() {
            take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;

            if opt(peek(literal("}"))).parse_next(parser_input)?.is_none() {
                let pat_offset = parser_input.current_token_start();
                let pat_content = read_until_close_brace(parser_input)?;
                value = Some(parse_pattern(pat_content, ts, pat_offset as u32)?);
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
                error = Some(parse_pattern(pat_content, ts, pat_offset as u32)?);
            }
            literal("}").parse_next(parser_input)?;

            let (nodes, _): (Vec<FragmentNode>, _) =
                repeat_till(0.., fragment_node_parser, peek(block_close_peek("await")))
                    .parse_next(parser_input)?;
            catch_fragment = Some(Fragment { nodes });
        }
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

    let loose = parser_input.state.loose;

    // Read snippet name with actual positions
    let name_start = parser_input.current_token_start();
    let name: &str = take_while(0.., |c: char| c.is_ascii_alphanumeric() || c == '_')
        .parse_next(parser_input)?;

    if name.is_empty() && !loose {
        return Err(winnow::error::ContextError::new());
    }

    let name_end = parser_input.current_token_start();
    let ident = Box::new(swc::Ident::new(
        name.into(),
        swc_common::Span::new(
            swc_common::BytePos(name_start as u32),
            swc_common::BytePos(name_end as u32),
        ),
        Default::default(),
    ));

    // Optional generic type params: <T extends string>
    let type_params = if parser_input.state.ts {
        let _: &str =
            take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;
        if !parser_input.input.is_empty()
            && parser_input.input.as_bytes()[0] == b'<'
        {
            // Find matching > handling nested <>, strings, etc.
            let remaining: &str = &parser_input.input;
            if let Some(end_offset) = find_matching_angle_bracket(remaining) {
                let content = &remaining[1..end_offset]; // between < and >
                let type_params_str = content.to_string();
                let _: &str = take(end_offset + 1).parse_next(parser_input)?;
                Some(type_params_str)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;

    // Parse parameters - in loose mode, ( might not be present
    let parameters = if opt(peek(literal("("))).parse_next(parser_input)?.is_some() {
        let paren_offset = parser_input.current_token_start();
        literal("(").parse_next(parser_input)?;
        let params_content = read_until_close_paren(parser_input)?;
        literal(")").parse_next(parser_input)?;
        take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;
        parse_param_list(params_content, parser_input.state.ts, paren_offset as u32)?
    } else if loose {
        vec![]
    } else {
        return Err(winnow::error::ContextError::new());
    };

    literal("}").parse_next(parser_input)?;

    // Use peek terminator so trailing text before {/snippet} is captured in the body
    let (nodes, _): (Vec<FragmentNode>, _) =
        repeat_till(0.., fragment_node_parser, peek(block_close_peek("snippet")))
            .parse_next(parser_input)?;

    // Consume {/snippet}
    block_close("snippet").parse_next(parser_input)?;

    let end = parser_input.previous_token_end();

    Ok(FragmentNode::SnippetBlock(SnippetBlock {
        span: Span::new(start, end),
        expression: ident,
        type_params,
        parameters,
        body: Fragment { nodes },
    }))
}

/// Find the position of the matching `>` for an opening `<` at position 0.
/// Handles nested `<>`, string literals, and escaped chars.
/// Returns the byte offset of the closing `>`, or None if not found.
fn find_matching_angle_bracket(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    if bytes.is_empty() || bytes[0] != b'<' {
        return None;
    }
    let mut depth: u32 = 1;
    let mut i = 1;

    while i < bytes.len() && depth > 0 {
        match bytes[i] {
            b'<' => {
                depth += 1;
                i += 1;
            }
            b'>' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
                i += 1;
            }
            b'"' | b'\'' | b'`' => {
                let quote = bytes[i];
                i += 1;
                while i < bytes.len() && bytes[i] != quote {
                    if bytes[i] == b'\\' {
                        i += 1;
                        if i >= bytes.len() {
                            break;
                        }
                    }
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1; // past closing quote
                }
            }
            _ => {
                i += 1;
            }
        }
    }
    None
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
