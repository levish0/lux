mod common;
use common::parse_nodes;

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
