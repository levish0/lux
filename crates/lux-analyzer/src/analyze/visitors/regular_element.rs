//! RegularElement visitor for analysis.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/RegularElement.js`
//!
//! This visitor handles validation and metadata collection for regular HTML elements.

use lux_ast::elements::RegularElement;
use lux_ast::node::AttributeNode;
use lux_utils::{is_mathml, is_svg};

use crate::analyze::analysis::RegularElementMeta;
use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;
use crate::analyze::visitors::shared::validate_element;

/// RegularElement visitor.
///
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/RegularElement.js`
pub fn visit_regular_element(
    node: &RegularElement<'_>,
    state: &mut AnalysisState<'_, '_>,
    path: &[NodeKind<'_>],
) {
    // validate_element(node, context);
    validate_element(&node.attributes, state, path);

    // check_element(node, context); - a11y checks
    // TODO: Implement a11y checks

    // node.metadata.has_spread = node.attributes.some(
    //     (attribute) => attribute.type === 'SpreadAttribute'
    // );
    let has_spread = node.attributes.iter().any(|attr| {
        matches!(attr, AttributeNode::SpreadAttribute(_))
    });

    // Determine if this is an SVG element
    let svg = is_svg_element(node.name, path);

    // Determine if this is a MathML element
    let mathml = is_mathml(node.name);

    // Store metadata
    let meta = state
        .analysis
        .regular_element_meta
        .entry(node.span.into())
        .or_insert_with(RegularElementMeta::default);
    meta.has_spread = has_spread;
    meta.svg = svg;
    meta.mathml = mathml;

    // Special case: <option> with single ExpressionTag child
    // if (
    //     node.name === 'option' &&
    //     node.fragment.nodes?.length === 1 &&
    //     node.fragment.nodes[0].type === 'ExpressionTag' &&
    //     !node.attributes.some(...)
    // ) { ... }
    if node.name == "option" {
        let nodes = &node.fragment.nodes;
        if nodes.len() == 1 {
            if let lux_ast::node::FragmentNode::ExpressionTag(expr_tag) = &nodes[0] {
                // Check if there's no value attribute
                let has_value_attr = node.attributes.iter().any(|attr| {
                    if let AttributeNode::Attribute(a) = attr {
                        a.name == "value"
                    } else {
                        false
                    }
                });

                if !has_value_attr {
                    meta.synthetic_value_node = Some(expr_tag.span.into());
                }
            }
        }
    }

    // TODO: Special case for textarea - moving children to value attribute
    // TODO: Special case for customizable select elements
    // TODO: HTML tree validation (parent/ancestor checks)
    // TODO: Self-closing tag validation for non-void elements
}

/// Check if an element name is an SVG element, considering context.
/// 'a' and 'title' can be SVG elements if inside SVG context.
fn is_svg_element(name: &str, path: &[NodeKind<'_>]) -> bool {
    // Direct SVG elements
    if is_svg(name) {
        return true;
    }

    // Special case: 'a' and 'title' can be SVG elements if inside SVG context
    if name == "a" || name == "title" {
        // Walk up the path to find SVG ancestor
        for ancestor in path.iter().rev() {
            if let NodeKind::RegularElement(ancestor_name) = ancestor {
                // If we find an SVG ancestor, this is an SVG element
                if is_svg(ancestor_name) {
                    return true;
                }
            }
        }
    }

    false
}
