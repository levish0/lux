mod common;
use common::parse_nodes;
use lux_parser::parse;
use oxc_allocator::Allocator;

#[test]
fn test_simple_text() {
    assert_eq!(parse_nodes("hello"), vec!["Text"]);
}

#[test]
fn test_expression_tag() {
    assert_eq!(parse_nodes("{name}"), vec!["ExpressionTag"]);
}

#[test]
fn test_element_with_text() {
    assert_eq!(parse_nodes("<div>hello</div>"), vec!["RegularElement"]);
}

#[test]
fn test_comment() {
    assert_eq!(parse_nodes("<!-- comment -->"), vec!["Comment"]);
}

#[test]
fn test_mixed_nodes() {
    assert_eq!(
        parse_nodes("hello {name} <div>world</div>"),
        vec!["Text", "ExpressionTag", "Text", "RegularElement"]
    );
}

#[test]
fn test_nested_blocks() {
    assert_eq!(
        parse_nodes("{#if cond}<div>{#each items as item}{item}{/each}</div>{/if}"),
        vec!["IfBlock"]
    );
}

#[test]
fn test_parses_textarea_boolean_and_expression_attributes() {
    let source = "<textarea readonly></textarea>\n<textarea autocomplete={'no'}></textarea>\n";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);

    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    assert_eq!(parsed.root.fragment.nodes.len(), 4);
}
