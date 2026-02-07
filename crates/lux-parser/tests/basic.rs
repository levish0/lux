mod common;
use common::parse_nodes;

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
