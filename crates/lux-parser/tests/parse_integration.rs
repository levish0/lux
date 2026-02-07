use lux_ast::template::root::FragmentNode;
use lux_parser::parse;
use oxc_allocator::Allocator;

fn parse_nodes(template: &str) -> Vec<String> {
    let allocator = Allocator::default();
    let result = parse(template, &allocator, false);
    result
        .root
        .fragment
        .nodes
        .iter()
        .map(node_type_name)
        .collect()
}

fn node_type_name(node: &FragmentNode) -> String {
    match node {
        FragmentNode::Text(_) => "Text".into(),
        FragmentNode::ExpressionTag(_) => "ExpressionTag".into(),
        FragmentNode::HtmlTag(_) => "HtmlTag".into(),
        FragmentNode::ConstTag(_) => "ConstTag".into(),
        FragmentNode::DebugTag(_) => "DebugTag".into(),
        FragmentNode::RenderTag(_) => "RenderTag".into(),
        FragmentNode::Comment(_) => "Comment".into(),
        FragmentNode::RegularElement(_) => "RegularElement".into(),
        FragmentNode::IfBlock(_) => "IfBlock".into(),
        FragmentNode::EachBlock(_) => "EachBlock".into(),
        FragmentNode::AwaitBlock(_) => "AwaitBlock".into(),
        FragmentNode::KeyBlock(_) => "KeyBlock".into(),
        FragmentNode::SnippetBlock(_) => "SnippetBlock".into(),
        _ => "Other".into(),
    }
}

#[test]
fn test_simple_text() {
    assert_eq!(parse_nodes("hello"), vec!["Text"]);
}

#[test]
fn test_expression_tag() {
    assert_eq!(parse_nodes("{name}"), vec!["ExpressionTag"]);
}

#[test]
fn test_html_tag() {
    assert_eq!(parse_nodes("{@html content}"), vec!["HtmlTag"]);
}

#[test]
fn test_element_with_text() {
    assert_eq!(parse_nodes("<div>hello</div>"), vec!["RegularElement"]);
}

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
fn test_await_block() {
    assert_eq!(
        parse_nodes("{#await promise}{:then value}done{:catch err}fail{/await}"),
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
fn test_const_tag() {
    assert_eq!(parse_nodes("{@const x = 42}"), vec!["ConstTag"]);
}

#[test]
fn test_debug_tag() {
    assert_eq!(parse_nodes("{@debug x}"), vec!["DebugTag"]);
}

#[test]
fn test_render_tag() {
    assert_eq!(parse_nodes("{@render header()}"), vec!["RenderTag"]);
}

#[test]
fn test_comment() {
    assert_eq!(parse_nodes("<!-- comment -->"), vec!["Comment"]);
}

#[test]
fn test_nested_blocks() {
    assert_eq!(
        parse_nodes("{#if cond}<div>{#each items as item}{item}{/each}</div>{/if}"),
        vec!["IfBlock"]
    );
}

// -- Grammar-based expression boundary tests --

#[test]
fn test_expression_complex() {
    assert_eq!(parse_nodes("{a + b * c}"), vec!["ExpressionTag"]);
}

#[test]
fn test_expression_ternary() {
    assert_eq!(parse_nodes("{a ? b : c}"), vec!["ExpressionTag"]);
}

#[test]
fn test_expression_function_call() {
    assert_eq!(parse_nodes("{foo(1, 2)}"), vec!["ExpressionTag"]);
}

#[test]
fn test_expression_object_literal() {
    assert_eq!(parse_nodes("{{ a: 1, b: 2 }}"), vec!["ExpressionTag"]);
}

#[test]
fn test_expression_arrow() {
    assert_eq!(parse_nodes("{() => 42}"), vec!["ExpressionTag"]);
}

#[test]
fn test_expression_template_literal() {
    assert_eq!(parse_nodes("{`hello ${name}`}"), vec!["ExpressionTag"]);
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
fn test_const_destructuring() {
    assert_eq!(parse_nodes("{@const { a, b } = obj}"), vec!["ConstTag"]);
}

#[test]
fn test_debug_multiple() {
    assert_eq!(parse_nodes("{@debug x, y, z}"), vec!["DebugTag"]);
}

#[test]
fn test_debug_empty() {
    assert_eq!(parse_nodes("{@debug}"), vec!["DebugTag"]);
}

#[test]
fn test_mixed_nodes() {
    assert_eq!(
        parse_nodes("hello {name} <div>world</div>"),
        vec!["Text", "ExpressionTag", "Text", "RegularElement"]
    );
}
