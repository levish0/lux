mod element;
mod node;

use lux_ast::template::root::{Fragment, FragmentNode};

pub(super) fn render_fragment(fragment: &Fragment<'_>, out: &mut String, has_dynamic: &mut bool) {
    let nodes = fragment.nodes.iter().collect::<Vec<_>>();
    render_fragment_nodes(&nodes, out, has_dynamic);
}

pub(super) fn render_fragment_nodes(
    nodes: &[&FragmentNode<'_>],
    out: &mut String,
    has_dynamic: &mut bool,
) {
    for node in nodes {
        node::render_node(node, out, has_dynamic);
    }
}
