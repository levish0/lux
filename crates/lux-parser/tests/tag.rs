mod common;
use common::parse_nodes;
use lux_ast::template::root::FragmentNode;
use lux_parser::parse;
use oxc_allocator::Allocator;
use oxc_ast::ast::BindingPattern;

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

#[test]
fn test_const_tag_id_is_binding_pattern() {
    let allocator = Allocator::default();
    let parsed = parse("{@const { a, b } = obj}", &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let node = parsed
        .root
        .fragment
        .nodes
        .first()
        .expect("expected one node");
    let FragmentNode::ConstTag(tag) = node else {
        panic!("expected ConstTag");
    };

    assert!(matches!(
        tag.declaration.id,
        BindingPattern::ObjectPattern(_)
    ));
}
