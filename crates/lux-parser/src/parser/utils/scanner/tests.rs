use super::*;

#[test]
fn test_scan_expression_stop_at_equals() {
    assert_eq!(scan_expression_boundary("x = 42}", &[b'=']), Some(2));
}

#[test]
fn test_scan_expression_stop_at_equals_nested() {
    assert_eq!(
        scan_expression_boundary("{ a = 1 } = obj}", &[b'=']),
        Some(10)
    );
}

#[test]
fn test_scan_expression_stop_at_comma() {
    assert_eq!(scan_expression_boundary("a, b)", &[b',', b')']), Some(1));
}

#[test]
fn test_scan_expression_stop_at_paren() {
    assert_eq!(scan_expression_boundary("x)", &[b')']), Some(1));
}

#[test]
fn test_scan_each_expression_end() {
    assert_eq!(scan_each_expression_boundary("items as item}"), Some(6));
}

#[test]
fn test_scan_each_expression_end_no_as() {
    assert_eq!(scan_each_expression_boundary("items}"), Some(5));
}

#[test]
fn test_scan_each_expression_end_complex() {
    assert_eq!(
        scan_each_expression_boundary("items.filter(x => x.ok) as item}"),
        Some(24)
    );
}

#[test]
fn test_find_matching_bracket_simple_braces() {
    assert_eq!(find_matching_bracket("{ a }", 1, '{'), Some(4));
}

#[test]
fn test_find_matching_bracket_nested_braces() {
    assert_eq!(find_matching_bracket("{ { a } }", 1, '{'), Some(8));
}

#[test]
fn test_find_matching_bracket_string_inside() {
    assert_eq!(find_matching_bracket("{ '}' }", 1, '{'), Some(6));
}

#[test]
fn test_find_matching_bracket_template_literal() {
    assert_eq!(find_matching_bracket("{ `}` }", 1, '{'), Some(6));
}

#[test]
fn test_find_matching_bracket_line_comment() {
    assert_eq!(find_matching_bracket("{ // }\n}", 1, '{'), Some(7));
}

#[test]
fn test_find_matching_bracket_block_comment() {
    assert_eq!(find_matching_bracket("{ /* } */ }", 1, '{'), Some(10));
}

#[test]
fn test_find_matching_bracket_unmatched() {
    assert_eq!(find_matching_bracket("{ a", 1, '{'), None);
}

#[test]
fn test_find_matching_bracket_parens() {
    assert_eq!(find_matching_bracket("(a + b)", 1, '('), Some(6));
}

#[test]
fn test_find_matching_bracket_nested_template_expression() {
    assert_eq!(find_matching_bracket("{ `${a}` }", 1, '{'), Some(9));
}
