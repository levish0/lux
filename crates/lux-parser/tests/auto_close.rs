mod common;
use common::{get_element_children, get_element_name};
use lux_parser::parse;
use oxc_allocator::Allocator;

#[test]
fn test_auto_close_li_sibling() {
    let allocator = Allocator::default();
    let result = parse("<ul><li>a<li>b</ul>", &allocator, false);
    let ul = &result.root.fragment.nodes[0];
    let children = get_element_children(ul);
    assert_eq!(children.len(), 2);
    assert_eq!(get_element_name(&children[0]), "li");
    assert_eq!(get_element_name(&children[1]), "li");
}

#[test]
fn test_auto_close_p_by_div() {
    let allocator = Allocator::default();
    let result = parse("<div><p>text<div>inner</div></div>", &allocator, false);
    let outer_div = &result.root.fragment.nodes[0];
    let children = get_element_children(outer_div);
    assert_eq!(children.len(), 2);
    assert_eq!(get_element_name(&children[0]), "p");
    assert_eq!(get_element_name(&children[1]), "div");
}

#[test]
fn test_auto_close_p_by_parent_closing() {
    let allocator = Allocator::default();
    let result = parse("<div><p></div>", &allocator, false);
    let outer_div = &result.root.fragment.nodes[0];
    let children = get_element_children(outer_div);
    assert_eq!(children.len(), 1);
    assert_eq!(get_element_name(&children[0]), "p");
}

#[test]
fn test_auto_close_table_elements() {
    let allocator = Allocator::default();
    let result = parse("<table><tr><td>a<td>b</tr></table>", &allocator, false);
    let table = &result.root.fragment.nodes[0];
    let tr = &get_element_children(table)[0];
    let tds = get_element_children(tr);
    assert_eq!(tds.len(), 2);
    assert_eq!(get_element_name(&tds[0]), "td");
    assert_eq!(get_element_name(&tds[1]), "td");
}

#[test]
fn test_normal_close_unchanged() {
    let allocator = Allocator::default();
    let result = parse("<div><b></b></div>", &allocator, false);
    let outer_div = &result.root.fragment.nodes[0];
    let children = get_element_children(outer_div);
    assert_eq!(children.len(), 1);
    assert_eq!(get_element_name(&children[0]), "b");
}
