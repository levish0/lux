mod common;
use common::parse_nodes;

#[test]
fn test_html_tag() {
    assert_eq!(parse_nodes("{@html content}"), vec!["HtmlTag"]);
}

#[test]
fn test_const_tag() {
    assert_eq!(parse_nodes("{@const x = 42}"), vec!["ConstTag"]);
}

#[test]
fn test_const_destructuring() {
    assert_eq!(parse_nodes("{@const { a, b } = obj}"), vec!["ConstTag"]);
}

#[test]
fn test_debug_tag() {
    assert_eq!(parse_nodes("{@debug x}"), vec!["DebugTag"]);
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
fn test_render_tag() {
    assert_eq!(parse_nodes("{@render header()}"), vec!["RenderTag"]);
}
