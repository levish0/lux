mod element;
mod node;

use lux_ast::template::root::{Fragment, FragmentNode};

use super::StaticRenderContext;

pub(super) fn render_fragment(
    fragment: &Fragment<'_>,
    out: &mut String,
    has_dynamic: &mut bool,
    context: &StaticRenderContext<'_>,
    select_value: Option<&str>,
) {
    let nodes = trim_edge_whitespace_nodes(fragment.nodes.iter().collect::<Vec<_>>());
    render_fragment_nodes(&nodes, out, has_dynamic, context, select_value);
}

pub(super) fn render_fragment_nodes(
    nodes: &[&FragmentNode<'_>],
    out: &mut String,
    has_dynamic: &mut bool,
    context: &StaticRenderContext<'_>,
    select_value: Option<&str>,
) {
    let trimmed = trim_edge_whitespace_nodes(nodes.to_vec());
    for node in &trimmed {
        node::render_node(node, out, has_dynamic, context, select_value);
    }
}

fn trim_edge_whitespace_nodes<'a>(nodes: Vec<&'a FragmentNode<'a>>) -> Vec<&'a FragmentNode<'a>> {
    let start = nodes
        .iter()
        .position(|node| !is_whitespace_text_node(node))
        .unwrap_or(nodes.len());
    let end = nodes
        .iter()
        .rposition(|node| !is_whitespace_text_node(node))
        .map(|index| index + 1)
        .unwrap_or(start);

    nodes[start..end].to_vec()
}

fn is_whitespace_text_node(node: &FragmentNode<'_>) -> bool {
    match node {
        FragmentNode::Text(text) => text.raw.trim().is_empty(),
        _ => false,
    }
}
