mod common;
use common::parse_nodes;
use lux_ast::template::root::FragmentNode;
use lux_parser::parse;
use oxc_allocator::Allocator;
use oxc_span::GetSpan;

#[test]
fn test_if_block() {
    assert_eq!(parse_nodes("{#if cond}yes{/if}"), vec!["IfBlock"]);
}

#[test]
fn test_if_else_block() {
    assert_eq!(parse_nodes("{#if cond}yes{:else}no{/if}"), vec!["IfBlock"]);
}

#[test]
fn test_if_else_if_block() {
    assert_eq!(
        parse_nodes("{#if a}1{:else if b}2{:else}3{/if}"),
        vec!["IfBlock"]
    );
}

#[test]
fn test_each_block() {
    assert_eq!(
        parse_nodes("{#each items as item}text{/each}"),
        vec!["EachBlock"]
    );
}

#[test]
fn test_each_with_index_and_key() {
    assert_eq!(
        parse_nodes("{#each items as item, i (item.id)}text{/each}"),
        vec!["EachBlock"]
    );
}

#[test]
fn test_each_with_else() {
    assert_eq!(
        parse_nodes("{#each items as item}{item}{:else}none{/each}"),
        vec!["EachBlock"]
    );
}

#[test]
fn test_each_complex_expression() {
    assert_eq!(
        parse_nodes("{#each items.filter(x => x.ok) as item}{item}{/each}"),
        vec!["EachBlock"]
    );
}

#[test]
fn test_each_object_destructuring() {
    assert_eq!(
        parse_nodes("{#each items as {id, name}}{id}{/each}"),
        vec!["EachBlock"]
    );
}

#[test]
fn test_each_array_destructuring() {
    assert_eq!(
        parse_nodes("{#each items as [a, b]}{a}{/each}"),
        vec!["EachBlock"]
    );
}

#[test]
fn test_each_context_pattern_span_is_preserved() {
    let source = "{#each items as {id, name = 'x'}}{id}{/each}";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);

    let FragmentNode::EachBlock(node) = &parsed.root.fragment.nodes[0] else {
        panic!("expected each block");
    };

    let context = node.context.as_ref().expect("expected each context");
    assert_eq!(
        &source[context.span().start as usize..context.span().end as usize],
        "{id, name = 'x'}"
    );
}

#[test]
fn test_await_block() {
    assert_eq!(
        parse_nodes("{#await promise}{:then value}done{:catch err}fail{/await}"),
        vec!["AwaitBlock"]
    );
}

#[test]
fn test_await_inline_then() {
    assert_eq!(
        parse_nodes("{#await promise then value}{value}{/await}"),
        vec!["AwaitBlock"]
    );
}

#[test]
fn test_await_inline_catch() {
    assert_eq!(
        parse_nodes("{#await promise catch err}{err}{/await}"),
        vec!["AwaitBlock"]
    );
}

#[test]
fn test_key_block() {
    assert_eq!(parse_nodes("{#key value}content{/key}"), vec!["KeyBlock"]);
}

#[test]
fn test_snippet_block() {
    assert_eq!(
        parse_nodes("{#snippet hello(name)}hi{/snippet}"),
        vec!["SnippetBlock"]
    );
}
