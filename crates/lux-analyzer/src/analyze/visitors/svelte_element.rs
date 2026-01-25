//! SvelteElement visitor for analysis.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/SvelteElement.js`

use lux_ast::attributes::{AttributeSequenceValue, AttributeValue};
use lux_ast::node::AttributeNode;
use lux_ast::elements::SvelteElement;

use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;
use crate::analyze::visitors::shared::{mark_subtree_dynamic, validate_element};

/// SvelteElement visitor.
///
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/SvelteElement.js`
pub fn visit_svelte_element(
    node: &SvelteElement<'_>,
    state: &mut AnalysisState<'_, '_>,
    path: &[NodeKind<'_>],
) {
    // Validate element attributes
    validate_element(&node.attributes, state, path);

    // TODO: check_element for a11y

    // Determine namespace from xmlns attribute or ancestors
    let mut is_svg = false;
    let mut is_mathml = false;

    // Check for xmlns attribute
    let xmlns = node.attributes.iter().find_map(|attr| {
        if let AttributeNode::Attribute(a) = attr {
            if a.name == "xmlns" {
                // Check if it's a text attribute (static value)
                if let AttributeValue::Sequence(seq) = &a.value {
                    if seq.len() == 1 {
                        if let AttributeSequenceValue::Text(t) = &seq[0] {
                            return Some(t.data);
                        }
                    }
                }
            }
        }
        None
    });

    if let Some(xmlns_value) = xmlns {
        is_svg = xmlns_value == "http://www.w3.org/2000/svg";
        is_mathml = xmlns_value == "http://www.w3.org/1998/Math/MathML";
    } else {
        // Walk up ancestors to determine namespace
        for ancestor in path.iter().rev() {
            match ancestor {
                NodeKind::Component(_)
                | NodeKind::SvelteComponent
                | NodeKind::SvelteFragment
                | NodeKind::SnippetBlock
                | NodeKind::Root => {
                    // Root element, or inside a slot or a snippet -> this resets the namespace
                    // TODO: Check options.namespace when options are available
                    is_svg = false;
                    is_mathml = false;
                    break;
                }
                NodeKind::SvelteElement => {
                    // Inherit from parent svelte:element
                    // Note: We'd need metadata tracking for this
                    break;
                }
                NodeKind::RegularElement(name) => {
                    if *name == "foreignObject" {
                        is_svg = false;
                        is_mathml = false;
                    }
                    // Otherwise inherit from parent - would need metadata
                    break;
                }
                _ => continue,
            }
        }
    }

    // Store metadata if needed
    let _ = (is_svg, is_mathml);
    // TODO: Store svelte:element metadata

    mark_subtree_dynamic(state.analysis, path);
}
