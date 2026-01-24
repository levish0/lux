use svelte_ast::blocks::{AwaitBlock, EachBlock, IfBlock, KeyBlock, SnippetBlock};
use svelte_ast::metadata::{ExpressionNodeMetadata, SnippetBlockMetadata};
use svelte_ast::node::FragmentNode;
use svelte_ast::root::Fragment;
use svelte_ast::span::Span;

use crate::error::ErrorKind;
use crate::parser::{AwaitPhase, ParseError, Parser, StackFrame};

use super::skip_to_closing_brace;

/// `{/...}` — close block
pub fn close(parser: &mut Parser) -> Result<(), ParseError> {
    let current_type = match parser.current() {
        Some(StackFrame::IfBlock { .. }) => "if",
        Some(StackFrame::EachBlock { .. }) => "each",
        Some(StackFrame::AwaitBlock { .. }) => "await",
        Some(StackFrame::KeyBlock { .. }) => "key",
        Some(StackFrame::SnippetBlock { .. }) => "snippet",
        _ => {
            if !parser.loose {
                return Err(parser.error(
                    ErrorKind::BlockUnexpectedClose,
                    parser.index,
                    "Unexpected block close".to_string(),
                ));
            }
            skip_to_closing_brace(parser);
            return Ok(());
        }
    };

    match current_type {
        "if" => close_if(parser)?,
        "each" => close_each(parser)?,
        "await" => close_await(parser)?,
        "key" => close_key(parser)?,
        "snippet" => close_snippet(parser)?,
        _ => skip_to_closing_brace(parser),
    }

    Ok(())
}

fn close_if(parser: &mut Parser) -> Result<(), ParseError> {
    let matched = parser.eat_required_with_loose("if", false)?;
    if !matched {
        // Loose mode: mismatched close, pop and retry
        let fragment_nodes = parser.fragments.pop().unwrap_or_default();
        let frame = parser.stack.pop();
        if let Some(StackFrame::IfBlock {
            start,
            test,
            consequent,
            elseif,
        }) = frame
        {
            let (cons, alt) = if let Some(cons_nodes) = consequent {
                (
                    Fragment { nodes: cons_nodes },
                    Some(Fragment {
                        nodes: fragment_nodes,
                    }),
                )
            } else {
                (
                    Fragment {
                        nodes: fragment_nodes,
                    },
                    None,
                )
            };
            parser.append(FragmentNode::IfBlock(IfBlock {
                span: Span::new(start, parser.index),
                elseif,
                test,
                consequent: cons,
                alternate: alt,
                metadata: ExpressionNodeMetadata::default(),
            }));
        }
        close(parser)?;
        return Ok(());
    }

    parser.allow_whitespace();
    parser.eat_required("}")?;

    // Unwind elseif chain from inside out
    let mut alternate: Option<Fragment> = None;

    loop {
        let frame = parser.stack.pop();

        let Some(StackFrame::IfBlock {
            start,
            elseif,
            test,
            consequent,
        }) = frame
        else {
            break;
        };

        let (cons_frag, alt_frag) = if let Some(cons_nodes) = consequent {
            if alternate.is_some() {
                // {:else if} — alternate provided by child, no fragment to pop
                (Fragment { nodes: cons_nodes }, alternate.take())
            } else {
                // {:else} — fragment_nodes is the alternate content
                let fragment_nodes = parser.fragments.pop().unwrap_or_default();
                (
                    Fragment { nodes: cons_nodes },
                    Some(Fragment {
                        nodes: fragment_nodes,
                    }),
                )
            }
        } else {
            // No :else — fragment_nodes is the consequent
            let fragment_nodes = parser.fragments.pop().unwrap_or_default();
            (
                Fragment {
                    nodes: fragment_nodes,
                },
                alternate.take(),
            )
        };

        let if_block = IfBlock {
            span: Span::new(start, parser.index),
            elseif,
            test,
            consequent: cons_frag,
            alternate: alt_frag,
            metadata: ExpressionNodeMetadata::default(),
        };

        if elseif {
            // Pop the placeholder fragment for the parent
            parser.fragments.pop();
            alternate = Some(Fragment {
                nodes: vec![FragmentNode::IfBlock(if_block)],
            });
        } else {
            parser.append(FragmentNode::IfBlock(if_block));
            break;
        }
    }

    Ok(())
}

fn close_each(parser: &mut Parser) -> Result<(), ParseError> {
    let matched = parser.eat_required_with_loose("each", false)?;
    if !matched {
        parser.fragments.pop();
        parser.stack.pop();
        close(parser)?;
        return Ok(());
    }
    parser.allow_whitespace();
    parser.eat_required("}")?;

    let fragment_nodes = parser.fragments.pop().unwrap_or_default();
    let frame = parser.stack.pop();

    if let Some(StackFrame::EachBlock {
        start,
        expression,
        context,
        index,
        key,
        body,
    }) = frame
    {
        let (body_frag, fallback) = if let Some(body_nodes) = body {
            (
                Fragment { nodes: body_nodes },
                Some(Fragment {
                    nodes: fragment_nodes,
                }),
            )
        } else {
            (
                Fragment {
                    nodes: fragment_nodes,
                },
                None,
            )
        };

        parser.append(FragmentNode::EachBlock(EachBlock {
            span: Span::new(start, parser.index),
            expression,
            context,
            body: body_frag,
            fallback,
            index,
            key,
        }));
    }

    Ok(())
}

fn close_await(parser: &mut Parser) -> Result<(), ParseError> {
    let matched = parser.eat_required_with_loose("await", false)?;
    if !matched {
        parser.fragments.pop();
        parser.stack.pop();
        close(parser)?;
        return Ok(());
    }
    parser.allow_whitespace();
    parser.eat_required("}")?;

    let fragment_nodes = parser.fragments.pop().unwrap_or_default();
    let frame = parser.stack.pop();

    if let Some(StackFrame::AwaitBlock {
        start,
        expression,
        value,
        error,
        pending,
        then,
        phase,
    }) = frame
    {
        let (pending_frag, then_frag, catch_frag) = match phase {
            AwaitPhase::Pending => (
                Some(Fragment {
                    nodes: fragment_nodes,
                }),
                then.map(|n| Fragment { nodes: n }),
                None,
            ),
            AwaitPhase::Then => (
                pending.map(|n| Fragment { nodes: n }),
                Some(Fragment {
                    nodes: fragment_nodes,
                }),
                None,
            ),
            AwaitPhase::Catch => (
                pending.map(|n| Fragment { nodes: n }),
                then.map(|n| Fragment { nodes: n }),
                Some(Fragment {
                    nodes: fragment_nodes,
                }),
            ),
        };

        parser.append(FragmentNode::AwaitBlock(AwaitBlock {
            span: Span::new(start, parser.index),
            expression,
            value,
            error,
            pending: pending_frag,
            then: then_frag,
            catch: catch_frag,
            metadata: ExpressionNodeMetadata::default(),
        }));
    }

    Ok(())
}

fn close_key<'a>(parser: &mut Parser<'a>) -> Result<(), ParseError> {
    let matched = parser.eat_required_with_loose("key", false)?;
    if !matched {
        parser.fragments.pop();
        parser.stack.pop();
        close(parser)?;
        return Ok(());
    }
    parser.allow_whitespace();
    parser.eat_required("}")?;

    let fragment_nodes = parser.fragments.pop().unwrap_or_default();
    let frame = parser.stack.pop();

    if let Some(StackFrame::KeyBlock { start, expression }) = frame {
        parser.append(FragmentNode::KeyBlock(KeyBlock {
            span: Span::new(start, parser.index),
            expression,
            fragment: Fragment {
                nodes: fragment_nodes,
            },
            metadata: ExpressionNodeMetadata::default(),
        }));
    }

    Ok(())
}

fn close_snippet<'a>(parser: &mut Parser<'a>) -> Result<(), ParseError> {
    let matched = parser.eat_required_with_loose("snippet", false)?;
    if !matched {
        parser.fragments.pop();
        parser.stack.pop();
        close(parser)?;
        return Ok(());
    }
    parser.allow_whitespace();
    parser.eat_required("}")?;

    let fragment_nodes = parser.fragments.pop().unwrap_or_default();
    let frame = parser.stack.pop();

    if let Some(StackFrame::SnippetBlock {
        start,
        expression,
        parameters,
        type_params,
    }) = frame
    {
        parser.append(FragmentNode::SnippetBlock(SnippetBlock {
            span: Span::new(start, parser.index),
            expression,
            parameters,
            type_params,
            body: Fragment {
                nodes: fragment_nodes,
            },
            metadata: SnippetBlockMetadata::default(),
        }));
    }

    Ok(())
}
