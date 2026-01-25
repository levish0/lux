use svelte_ast::blocks::{AwaitBlock, EachBlock, IfBlock, KeyBlock, SnippetBlock};
use svelte_ast::metadata::{ExpressionNodeMetadata, SnippetBlockMetadata};
use svelte_ast::node::FragmentNode;
use svelte_ast::root::Fragment;
use svelte_ast::span::Span;

use crate::error::ErrorKind;
use crate::parser::{AwaitPhase, ParseError, Parser, StackFrame};

/// `{/...}` — close block
/// Reference: tag.js close() function (lines 545-613)
pub fn close(parser: &mut Parser) -> Result<(), ParseError> {
    let start = parser.index - 1; // position after {/

    // Reference pattern: check current block type and try to match
    // If not matched, pop current and recurse
    loop {
        let matched = match parser.current() {
            Some(StackFrame::IfBlock { .. }) => {
                let m = parser.eat_required_with_loose("if", false)?;
                if m {
                    // Matched: do full IfBlock close logic
                    parser.allow_whitespace();
                    parser.eat_required("}")?;
                    close_if_matched(parser);
                    return Ok(());
                }
                false
            }
            Some(StackFrame::EachBlock { .. }) => {
                let m = parser.eat_required_with_loose("each", false)?;
                if m {
                    parser.allow_whitespace();
                    parser.eat_required("}")?;
                    close_each_matched(parser);
                    return Ok(());
                }
                false
            }
            Some(StackFrame::AwaitBlock { .. }) => {
                let m = parser.eat_required_with_loose("await", false)?;
                if m {
                    parser.allow_whitespace();
                    parser.eat_required("}")?;
                    close_await_matched(parser);
                    return Ok(());
                }
                false
            }
            Some(StackFrame::KeyBlock { .. }) => {
                let m = parser.eat_required_with_loose("key", false)?;
                if m {
                    parser.allow_whitespace();
                    parser.eat_required("}")?;
                    close_key_matched(parser);
                    return Ok(());
                }
                false
            }
            Some(StackFrame::SnippetBlock { .. }) => {
                let m = parser.eat_required_with_loose("snippet", false)?;
                if m {
                    parser.allow_whitespace();
                    parser.eat_required("}")?;
                    close_snippet_matched(parser);
                    return Ok(());
                }
                false
            }
            // Element types: in loose mode, set matched = false
            Some(StackFrame::RegularElement { .. })
            | Some(StackFrame::Component { .. })
            | Some(StackFrame::SvelteElement { .. })
            | Some(StackFrame::SvelteComponent { .. })
            | Some(StackFrame::SvelteSelf { .. })
            | Some(StackFrame::SvelteHead { .. })
            | Some(StackFrame::SvelteBody { .. })
            | Some(StackFrame::SvelteWindow { .. })
            | Some(StackFrame::SvelteDocument { .. })
            | Some(StackFrame::SvelteFragment { .. })
            | Some(StackFrame::SvelteOptions { .. })
            | Some(StackFrame::TitleElement { .. })
            | Some(StackFrame::SlotElement { .. })
            | Some(StackFrame::SvelteBoundary { .. }) => {
                if parser.loose {
                    false
                } else {
                    return Err(parser.error(
                        ErrorKind::BlockUnexpectedClose,
                        parser.index,
                        "Unexpected block close".to_string(),
                    ));
                }
            }
            None => {
                return Err(parser.error(
                    ErrorKind::BlockUnexpectedClose,
                    parser.index,
                    "Unexpected block close".to_string(),
                ));
            }
        };

        // If not matched, pop current frame and recurse
        // Reference: block.end = start - 1; parser.pop(); close(parser);
        if !matched {
            let end = start - 1; // end before the {/
            let fragment_nodes = parser.fragments.pop().unwrap_or_default();
            let frame = parser.stack.pop();

            if let Some(frame) = frame {
                // Convert frame to node and append to parent
                if let Some(node) = parser.frame_to_node_loose(frame, end, fragment_nodes) {
                    parser.append(node);
                }
            }
            // Continue loop to try close again with new current
            continue;
        }

        break;
    }

    Ok(())
}

/// Close IfBlock when "if" keyword matched
fn close_if_matched(parser: &mut Parser) {
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
}

/// Close EachBlock when "each" keyword matched
fn close_each_matched(parser: &mut Parser) {
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
}

/// Close AwaitBlock when "await" keyword matched
fn close_await_matched(parser: &mut Parser) {
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
}

/// Close KeyBlock when "key" keyword matched
fn close_key_matched(parser: &mut Parser) {
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
}

/// Close SnippetBlock when "snippet" keyword matched
fn close_snippet_matched(parser: &mut Parser) {
    let fragment_nodes = parser.fragments.pop().unwrap_or_default();
    let frame = parser.stack.pop();

    if let Some(StackFrame::SnippetBlock {
        start,
        expression,
        parameters,
        rest,
        type_params,
    }) = frame
    {
        parser.append(FragmentNode::SnippetBlock(SnippetBlock {
            span: Span::new(start, parser.index),
            expression,
            parameters,
            rest,
            type_params,
            body: Fragment {
                nodes: fragment_nodes,
            },
            metadata: SnippetBlockMetadata::default(),
        }));
    }
}
