use lux_ast::template::attribute::AttributeNode;
use lux_ast::template::root::Fragment;
use lux_utils::elements::is_void;

use crate::js::template::attribute::render_static_attribute;

use super::render_fragment;

pub(super) fn render_regular_element(
    name: &str,
    attributes: &[AttributeNode<'_>],
    children: &Fragment<'_>,
    out: &mut String,
    has_dynamic: &mut bool,
) {
    out.push('<');
    out.push_str(name);

    for attribute in attributes {
        if let Some(serialized) = render_static_attribute(attribute, has_dynamic) {
            out.push(' ');
            out.push_str(&serialized);
        }
    }

    out.push('>');

    if !is_void(name) {
        render_fragment(children, out, has_dynamic);
        out.push_str("</");
        out.push_str(name);
        out.push('>');
    }
}
