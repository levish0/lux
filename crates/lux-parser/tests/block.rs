mod common;
use common::parse_nodes;
use lux_ast::template::block::EachBlock;
use lux_ast::template::root::FragmentNode;
use lux_parser::{parse, parse_with_options, ParseOptions};
use oxc_allocator::Allocator;
use oxc_ast::ast::{BindingPattern, Expression};
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
fn test_each_with_index_and_key_in_typescript_mode_preserves_header_parts() {
    let source = "{#each text as line, i (i)}{line}{/each}";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, true);

    let FragmentNode::EachBlock(EachBlock {
        expression,
        context,
        index,
        key,
        ..
    }) = &parsed.root.fragment.nodes[0]
    else {
        panic!("expected each block");
    };

    let Expression::Identifier(expression_ident) = expression else {
        panic!("expected identifier expression");
    };
    assert_eq!(expression_ident.name.as_str(), "text");

    let Some(BindingPattern::BindingIdentifier(context_ident)) = context else {
        panic!("expected identifier context");
    };
    assert_eq!(context_ident.name.as_str(), "line");

    assert_eq!(*index, Some("i"));

    let Some(Expression::Identifier(key_ident)) = key else {
        panic!("expected identifier key");
    };
    assert_eq!(key_ident.name.as_str(), "i");
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
fn test_await_clause_binding_pattern_span_is_preserved() {
    let source = "{#await promise then { value = 1 }}{/await}";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);

    let FragmentNode::AwaitBlock(node) = &parsed.root.fragment.nodes[0] else {
        panic!("expected await block");
    };

    let value = node.value.as_ref().expect("expected then binding");
    assert_eq!(
        &source[value.span().start as usize..value.span().end as usize],
        "{ value = 1 }"
    );
}

#[test]
fn test_snippet_parameter_pattern_spans_are_preserved() {
    let source = "{#snippet demo({ id, name = 'x' }, item = defaultValue)}{/snippet}";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);

    let FragmentNode::SnippetBlock(node) = &parsed.root.fragment.nodes[0] else {
        panic!("expected snippet block");
    };

    assert_eq!(node.parameters.len(), 2);
    assert_eq!(
        &source[node.parameters[0].span().start as usize..node.parameters[0].span().end as usize],
        "{ id, name = 'x' }"
    );
    assert_eq!(
        &source[node.parameters[1].span().start as usize..node.parameters[1].span().end as usize],
        "item = defaultValue"
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

#[test]
fn test_snippet_rest_parameter_span_is_recorded() {
    let source = "{#snippet demo(...args)}{/snippet}";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);

    let FragmentNode::SnippetBlock(node) = &parsed.root.fragment.nodes[0] else {
        panic!("expected snippet block");
    };

    assert_eq!(node.rest_parameter_spans.len(), 1);
    let span = node.rest_parameter_spans[0];
    assert_eq!(&source[span.start as usize..span.end as usize], "...args");
}

#[test]
fn test_loose_invalid_each_key_recovers_empty_identifier() {
    let source = "{#each array as item (item.)}{/each}";
    let allocator = Allocator::default();
    let parsed = parse_with_options(
        source,
        &allocator,
        ParseOptions {
            ts: false,
            loose: true,
        },
    );

    let FragmentNode::EachBlock(node) = &parsed.root.fragment.nodes[0] else {
        panic!("expected each block");
    };

    let Some(Expression::Identifier(key)) = &node.key else {
        panic!("expected recovered key identifier");
    };
    assert_eq!(key.name.as_str(), "");
    assert_eq!(&source[key.span.start as usize..key.span.end as usize], "item.");
}

#[test]
fn test_loose_invalid_each_expression_recovers_empty_identifier() {
    let source = "{#each obj. as item}{/each}";
    let allocator = Allocator::default();
    let parsed = parse_with_options(
        source,
        &allocator,
        ParseOptions {
            ts: false,
            loose: true,
        },
    );

    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);

    let FragmentNode::EachBlock(node) = &parsed.root.fragment.nodes[0] else {
        panic!("expected each block");
    };

    let Expression::Identifier(expression) = &node.expression else {
        panic!("expected recovered each expression identifier");
    };
    assert_eq!(expression.name.as_str(), "");
    assert_eq!(&source[expression.span.start as usize..expression.span.end as usize], "obj.");
}

#[test]
fn test_loose_unclosed_component_open_tag_recovers_without_consuming_parent_close() {
    let source = "<div>\n\t<Comp foo={bar}\n</div>";
    let allocator = Allocator::default();
    let parsed = parse_with_options(
        source,
        &allocator,
        ParseOptions {
            ts: false,
            loose: true,
        },
    );

    let FragmentNode::RegularElement(div) = &parsed.root.fragment.nodes[0] else {
        panic!("expected outer div");
    };
    let FragmentNode::Component(component) = &div.fragment.nodes[1] else {
        panic!("expected loose component child");
    };

    assert_eq!(component.name, "Comp");
    assert!(component.fragment.nodes.is_empty());
    assert_eq!(
        &source[component.span.start as usize..component.span.end as usize],
        "<Comp foo={bar}\n"
    );
}

#[test]
fn test_loose_eof_regular_open_tag_recovers() {
    let source = "<div foo={bar}";
    let allocator = Allocator::default();
    let parsed = parse_with_options(
        source,
        &allocator,
        ParseOptions {
            ts: false,
            loose: true,
        },
    );

    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);

    let FragmentNode::RegularElement(node) = &parsed.root.fragment.nodes[0] else {
        panic!("expected regular element");
    };

    assert_eq!(node.name, "div");
    assert!(node.fragment.nodes.is_empty());
    assert_eq!(&source[node.span.start as usize..node.span.end as usize], "<div foo={bar}");
}

#[test]
fn test_loose_eof_unclosed_tag_recovers() {
    let source = "<open-ended";
    let allocator = Allocator::default();
    let parsed = parse_with_options(
        source,
        &allocator,
        ParseOptions {
            ts: false,
            loose: true,
        },
    );

    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);

    let FragmentNode::RegularElement(node) = &parsed.root.fragment.nodes[0] else {
        panic!("expected regular element");
    };

    assert_eq!(node.name, "open-ended");
    assert!(node.fragment.nodes.is_empty());
    assert_eq!(&source[node.span.start as usize..node.span.end as usize], "<open-ended");
}

#[test]
fn test_loose_unclosed_component_tag_inside_parent_recovers() {
    let source = "<div>\n\t<Comp>\n</div>";
    let allocator = Allocator::default();
    let parsed = parse_with_options(
        source,
        &allocator,
        ParseOptions {
            ts: false,
            loose: true,
        },
    );

    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    assert_eq!(parsed.root.fragment.nodes.len(), 1);
}

#[test]
fn test_loose_unclosed_regular_tag_inside_if_recovers() {
    let source = "{#if foo}\n\t<div>\n{/if}";
    let allocator = Allocator::default();
    let parsed = parse_with_options(
        source,
        &allocator,
        ParseOptions {
            ts: false,
            loose: true,
        },
    );

    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    assert_eq!(parsed.root.fragment.nodes.len(), 1);
}

#[test]
fn test_loose_unclosed_open_tag_inside_nested_if_recovers() {
    let source = "{#if foo}\n\t<Comp foo={bar}\n\t{#if bar}\n\t\t{bar}\n\t{/if}\n{/if}";
    let allocator = Allocator::default();
    let parsed = parse_with_options(
        source,
        &allocator,
        ParseOptions {
            ts: false,
            loose: true,
        },
    );

    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    assert_eq!(parsed.root.fragment.nodes.len(), 1);
}

#[test]
fn test_loose_invalid_expression_sample_parses_without_fatal_error() {
    let source = "<div {}></div>\n<div foo={}></div>\n\n<div foo={a.}></div>\n<div foo={'hi}'.}></div>\n<Component onclick={() => x.} />\n\n<input bind:value={a.} />\n\nasd{a.}asd\n{foo[bar.]}\n\n{#if x.}{/if}\n\n{#each array as item (item.)}{/each}\n\n{#each obj. as item}{/each}\n\n{#await x.}{/await}\n\n{#await x. then y}{/await}\n\n{#await x. catch y}{/await}";
    let allocator = Allocator::default();
    let parsed = parse_with_options(
        source,
        &allocator,
        ParseOptions {
            ts: false,
            loose: true,
        },
    );

    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    assert_eq!(parsed.root.fragment.nodes.len(), 27);
}

#[test]
fn test_loose_unclosed_open_tag_sample_parses_without_fatal_error() {
    let source = "<div>\n\t<Comp foo={bar}\n</div>\n\n<div>\n\t<span foo={bar}\n</div>\n\n{#if foo}\n\t<Comp foo={bar}\n{/if}\n\n{#if foo}\n\t<Comp foo={bar}\n\t{#if bar}\n\t\t{bar}\n\t{/if}\n{/if}\n\n<div foo={bar}";
    let allocator = Allocator::default();
    let parsed = parse_with_options(
        source,
        &allocator,
        ParseOptions {
            ts: false,
            loose: true,
        },
    );

    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    assert_eq!(parsed.root.fragment.nodes.len(), 9);
}

#[test]
fn test_loose_unclosed_tag_sample_parses_without_fatal_error() {
    let source = "<div>\n\t<Comp>\n</div>\n\n<div>\n\t<Comp foo={bar}\n</div>\n\n<div>\n\t<span\n</div>\n\n<div>\n\t<Comp.\n</div>\n\n<div>\n\t<comp.\n</div>\n\n{#if foo}\n\t<div>\n{/if}\n\n{#if foo}\n\t<Comp foo={bar}\n{/if}\n\n<div>\n<p>hi</p>\n\n<open-ended";
    let allocator = Allocator::default();
    let parsed = parse_with_options(
        source,
        &allocator,
        ParseOptions {
            ts: false,
            loose: true,
        },
    );

    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    assert_eq!(parsed.root.fragment.nodes.len(), 15);
}
