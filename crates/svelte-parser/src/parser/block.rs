use svelte_ast::blocks::{AwaitBlock, EachBlock, IfBlock, KeyBlock, SnippetBlock};
use svelte_ast::node::FragmentNode;
use svelte_ast::root::Fragment;
use svelte_ast::span::Span;
use swc_ecma_ast as swc;
use winnow::combinator::{peek, repeat_till};
use winnow::prelude::*;
use winnow::stream::Location;
use winnow::token::{any, literal, take_while};
use winnow::Result as ParseResult;

use super::ParserInput;
use super::fragment::fragment_node_parser;

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
    // whitespace before test expression
    take_while(1.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;

    // read test expression up to }
    let test = read_expression_until_close(parser_input)?;
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
    // Check if we have {:else if ...} or {:else} or {/if}
    let peeked: ParseResult<&str> = peek(literal("{:else")).parse_next(parser_input);
    if peeked.is_err() {
        return Ok(None);
    }

    literal("{:else").parse_next(parser_input)?;

    // Check for {:else if ...} vs {:else}
    let ws: &str =
        take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;
    let is_else_if: ParseResult<&str> = peek(literal("if")).parse_next(parser_input);

    if is_else_if.is_ok() && !ws.is_empty() {
        // {:else if test}...
        literal("if").parse_next(parser_input)?;
        take_while(1.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;

        let start = parser_input.current_token_start();
        // We need start to be from {:else if - back up
        // Actually, for the nested IfBlock, start should be from {#if equivalent
        let nested_start = start - "{:else if ".len(); // approximate

        let test = read_expression_until_close(parser_input)?;
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

        let (nodes, _): (Vec<FragmentNode>, _) = repeat_till(
            0..,
            fragment_node_parser,
            peek(block_close_peek("if")),
        )
        .parse_next(parser_input)?;

        block_close("if").parse_next(parser_input)?;

        Ok(Some(Fragment { nodes }))
    }
}

/// Parse `{#each expr as item, index (key)}...{:else}...{/each}`
fn each_block_parser(parser_input: &mut ParserInput, start: usize) -> ParseResult<FragmentNode> {
    take_while(1.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;

    // Read expression until " as " keyword
    let expr_text = read_until_keyword(parser_input, " as ")?;
    let expression = swc_parse_expr_from_str(&expr_text, parser_input.state.ts)?;

    literal(" as ").parse_next(parser_input)?;
    take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;

    // Read context pattern until , or ( or }
    let context_text = read_until_any(parser_input, &[',', '(', '}'])?;
    let context = if context_text.trim().is_empty() {
        None
    } else {
        Some(swc_parse_pattern(&context_text, parser_input.state.ts)?)
    };

    // Optional index
    let index = if peek(any).parse_next(parser_input)? == ',' {
        any.parse_next(parser_input)?; // consume ,
        take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;
        let idx: &str =
            take_while(1.., |c: char| c.is_ascii_alphanumeric() || c == '_')
                .parse_next(parser_input)?;
        take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;
        Some(idx.to_string())
    } else {
        None
    };

    // Optional key
    let key = if peek(any).parse_next(parser_input)? == '(' {
        any.parse_next(parser_input)?; // consume (
        let key_expr = read_expression_until_char(parser_input, ')')?;
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
    let has_else: ParseResult<&str> = peek(literal("{:else}")).parse_next(parser_input);
    let fallback = if has_else.is_ok() {
        literal("{:else}").parse_next(parser_input)?;
        let (nodes, _): (Vec<FragmentNode>, _) = repeat_till(
            0..,
            fragment_node_parser,
            peek(block_close_peek("each")),
        )
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

    let expression = read_expression_until_close(parser_input)?;
    literal("}").parse_next(parser_input)?;

    let (nodes, _): (Vec<FragmentNode>, _) = repeat_till(
        0..,
        fragment_node_parser,
        block_close("key"),
    )
    .parse_next(parser_input)?;

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

    let expression = read_expression_until_close(parser_input)?;
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
    let has_then: ParseResult<&str> = peek(literal("{:then")).parse_next(parser_input);
    if has_then.is_ok() {
        literal("{:then").parse_next(parser_input)?;
        take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;
        if peek(any).parse_next(parser_input)? != '}' {
            let pat_text = read_until_char(parser_input, '}')?;
            value = Some(swc_parse_pattern(&pat_text, parser_input.state.ts)?);
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
    let has_catch: ParseResult<&str> = peek(literal("{:catch")).parse_next(parser_input);
    if has_catch.is_ok() {
        literal("{:catch").parse_next(parser_input)?;
        take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;
        if peek(any).parse_next(parser_input)? != '}' {
            let pat_text = read_until_char(parser_input, '}')?;
            error = Some(swc_parse_pattern(&pat_text, parser_input.state.ts)?);
        }
        literal("}").parse_next(parser_input)?;

        let (nodes, _): (Vec<FragmentNode>, _) = repeat_till(
            0..,
            fragment_node_parser,
            peek(block_close_peek("await")),
        )
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
    let ident = Box::new(swc::Ident::new(name.into(), swc_common::DUMMY_SP, Default::default()));

    // Parse parameters
    literal("(").parse_next(parser_input)?;
    let params_text = read_until_char(parser_input, ')')?;
    literal(")").parse_next(parser_input)?;
    take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;
    literal("}").parse_next(parser_input)?;

    let parameters = parse_param_list(&params_text, parser_input.state.ts)?;

    let (nodes, _): (Vec<FragmentNode>, _) = repeat_till(
        0..,
        fragment_node_parser,
        block_close("snippet"),
    )
    .parse_next(parser_input)?;

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

/// Read expression text (without braces) until `}`, without consuming the `}`.
fn read_expression_until_close(parser_input: &mut ParserInput) -> ParseResult<Box<swc::Expr>> {
    let text = read_until_close_brace(parser_input)?;
    swc_parse_expr_from_str(&text, parser_input.state.ts)
}

fn read_expression_until_char(
    parser_input: &mut ParserInput,
    end_char: char,
) -> ParseResult<Box<swc::Expr>> {
    let text = read_until_char(parser_input, end_char)?;
    swc_parse_expr_from_str(&text, parser_input.state.ts)
}

/// Read text until `}` without consuming it. Handles nested braces.
fn read_until_close_brace(parser_input: &mut ParserInput) -> ParseResult<String> {
    let mut buf = String::new();
    let mut depth: u32 = 0;
    loop {
        let c = peek(any).parse_next(parser_input)?;
        if c == '}' && depth == 0 {
            break;
        }
        let c: char = any.parse_next(parser_input)?;
        buf.push(c);
        match c {
            '{' => depth += 1,
            '}' => depth -= 1,
            '"' | '\'' => {
                collect_string_inline(parser_input, c, &mut buf)?;
            }
            '`' => {
                collect_template_inline(parser_input, &mut buf)?;
            }
            _ => {}
        }
    }
    Ok(buf)
}

/// Read text until a specific character without consuming it.
fn read_until_char(parser_input: &mut ParserInput, end: char) -> ParseResult<String> {
    let mut buf = String::new();
    loop {
        let c = peek(any).parse_next(parser_input)?;
        if c == end {
            break;
        }
        let c: char = any.parse_next(parser_input)?;
        buf.push(c);
    }
    Ok(buf)
}

/// Read text until a keyword appears (not inside strings/braces).
fn read_until_keyword(parser_input: &mut ParserInput, keyword: &str) -> ParseResult<String> {
    let mut buf = String::new();
    let kw_len = keyword.len();
    loop {
        if buf.len() >= kw_len - 1 {
            let check: ParseResult<&str> =
                peek(literal(keyword)).parse_next(parser_input);
            if check.is_ok() {
                break;
            }
        }
        let c: char = any.parse_next(parser_input)?;
        buf.push(c);
    }
    Ok(buf)
}

/// Read text until one of the given characters.
fn read_until_any(parser_input: &mut ParserInput, chars: &[char]) -> ParseResult<String> {
    let text: &str = take_while(0.., |c: char| !chars.contains(&c)).parse_next(parser_input)?;
    Ok(text.trim().to_string())
}

fn collect_string_inline(
    parser_input: &mut ParserInput,
    quote: char,
    out: &mut String,
) -> ParseResult<()> {
    loop {
        let c: char = any.parse_next(parser_input)?;
        out.push(c);
        if c == quote {
            return Ok(());
        }
        if c == '\\' {
            let esc: char = any.parse_next(parser_input)?;
            out.push(esc);
        }
    }
}

fn collect_template_inline(parser_input: &mut ParserInput, out: &mut String) -> ParseResult<()> {
    loop {
        let c: char = any.parse_next(parser_input)?;
        out.push(c);
        match c {
            '`' => return Ok(()),
            '\\' => {
                let esc: char = any.parse_next(parser_input)?;
                out.push(esc);
            }
            _ => {}
        }
    }
}

/// Parser that matches `{:...}` or `{/name}`
fn block_continuation_or_close<'a>(
    name: &'a str,
) -> impl FnMut(&mut ParserInput<'_>) -> ParseResult<()> + 'a {
    move |input: &mut ParserInput| {
        let r1: ParseResult<&str> = peek(literal("{:")).parse_next(input);
        if r1.is_ok() {
            return Ok(());
        }
        let r2: ParseResult<()> = peek(block_close_peek(name)).parse_next(input);
        if r2.is_ok() {
            Ok(())
        } else {
            Err(winnow::error::ContextError::new())
        }
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

// --- SWC helpers ---

fn swc_parse_expr_from_str(source: &str, ts: bool) -> ParseResult<Box<swc::Expr>> {
    use swc_common::input::StringInput;
    use swc_common::BytePos;
    use swc_ecma_parser::{EsSyntax, Syntax, TsSyntax};

    let trimmed = source.trim();
    let syntax = if ts {
        Syntax::Typescript(TsSyntax {
            tsx: true,
            ..Default::default()
        })
    } else {
        Syntax::Es(EsSyntax {
            jsx: true,
            ..Default::default()
        })
    };

    let input = StringInput::new(trimmed, BytePos(0), BytePos(trimmed.len() as u32));
    let mut parser = swc_ecma_parser::Parser::new(syntax, input, None);

    parser.parse_expr().map_err(|e| {
        e.into_diagnostic(&swc_common::errors::Handler::with_emitter(
            true,
            false,
            Box::new(swc_common::errors::EmitterWriter::new(
                Box::new(std::io::sink()),
                None,
                false,
                false,
            )),
        ))
        .cancel();
        winnow::error::ContextError::new()
    })
}

fn swc_parse_pattern(source: &str, ts: bool) -> ParseResult<Box<swc::Pat>> {
    // Parse as expression first, then convert to pattern
    let expr = swc_parse_expr_from_str(source, ts)?;
    // Simple conversion: Ident → BindingIdent, ArrayExpr → ArrayPat, ObjectExpr → ObjectPat
    Ok(Box::new(expr_to_pat(*expr)))
}

fn expr_to_pat(expr: swc::Expr) -> swc::Pat {
    match expr {
        swc::Expr::Ident(id) => swc::Pat::Ident(swc::BindingIdent {
            id,
            type_ann: None,
        }),
        swc::Expr::Array(arr) => swc::Pat::Array(swc::ArrayPat {
            span: arr.span,
            elems: arr
                .elems
                .into_iter()
                .map(|e| e.map(|e| expr_to_pat(*e.expr)))
                .collect(),
            optional: false,
            type_ann: None,
        }),
        swc::Expr::Object(obj) => swc::Pat::Object(swc::ObjectPat {
            span: obj.span,
            props: obj
                .props
                .into_iter()
                .filter_map(|p| match p {
                    swc::PropOrSpread::Prop(prop) => match *prop {
                        swc::Prop::Shorthand(id) => {
                            Some(swc::ObjectPatProp::Assign(swc::AssignPatProp {
                                span: id.span,
                                key: swc::BindingIdent {
                                    id,
                                    type_ann: None,
                                },
                                value: None,
                            }))
                        }
                        swc::Prop::KeyValue(kv) => {
                            Some(swc::ObjectPatProp::KeyValue(swc::KeyValuePatProp {
                                key: kv.key,
                                value: Box::new(expr_to_pat(*kv.value)),
                            }))
                        }
                        _ => None,
                    },
                    swc::PropOrSpread::Spread(s) => {
                        Some(swc::ObjectPatProp::Rest(swc::RestPat {
                            span: s.dot3_token,
                            dot3_token: s.dot3_token,
                            arg: Box::new(expr_to_pat(*s.expr)),
                            type_ann: None,
                        }))
                    }
                })
                .collect(),
            optional: false,
            type_ann: None,
        }),
        _ => swc::Pat::Expr(Box::new(expr)),
    }
}

fn parse_param_list(source: &str, ts: bool) -> ParseResult<Vec<swc::Pat>> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Ok(vec![]);
    }
    // Split by comma (simple approach, doesn't handle commas in nested structures)
    // For robustness, parse as arrow function params
    let wrapper = format!("({}) => {{}}", trimmed);
    let expr = swc_parse_expr_from_str(&wrapper, ts)?;
    match *expr {
        swc::Expr::Arrow(arrow) => Ok(arrow.params),
        _ => Ok(vec![]),
    }
}
