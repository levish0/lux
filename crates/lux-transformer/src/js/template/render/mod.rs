mod element;
mod node;

use lux_ast::template::root::Fragment;

pub(super) fn render_fragment(fragment: &Fragment<'_>, out: &mut String, has_dynamic: &mut bool) {
    for node in &fragment.nodes {
        node::render_node(node, out, has_dynamic);
    }
}
