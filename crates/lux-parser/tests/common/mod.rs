#![allow(dead_code)]

use lux_ast::template::root::FragmentNode;
use lux_parser::parse;
use oxc_allocator::Allocator;

pub fn parse_nodes(template: &str) -> Vec<String> {
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

pub fn node_type_name(node: &FragmentNode) -> String {
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

pub fn get_element_children<'a>(node: &'a FragmentNode<'a>) -> &'a [FragmentNode<'a>] {
    match node {
        FragmentNode::RegularElement(el) => &el.fragment.nodes,
        _ => &[],
    }
}

pub fn get_element_name<'a>(node: &'a FragmentNode<'a>) -> &'a str {
    match node {
        FragmentNode::RegularElement(el) => el.name,
        _ => "",
    }
}
