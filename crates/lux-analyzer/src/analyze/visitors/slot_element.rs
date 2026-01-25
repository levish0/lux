//! SlotElement visitor for analysis.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/SlotElement.js`

use lux_ast::attributes::{AttributeSequenceValue, AttributeValue};
use lux_ast::elements::SlotElement;
use lux_ast::node::AttributeNode;

use crate::analyze::errors;
use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;
use crate::analyze::visitors::shared::mark_subtree_dynamic;
use crate::analyze::warnings;

/// SlotElement visitor.
///
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/SlotElement.js`
pub fn visit_slot_element(
    node: &SlotElement<'_>,
    state: &mut AnalysisState<'_, '_>,
    path: &[NodeKind<'_>],
) {
    // if (context.state.analysis.runes && !context.state.analysis.custom_element) {
    //     w.slot_element_deprecated(node);
    // }
    if state.analysis.runes && !state.analysis.custom_element {
        state
            .analysis
            .warning(warnings::slot_element_deprecated(node.span.into()));
    }

    // mark_subtree_dynamic(context.path);
    mark_subtree_dynamic(state.analysis, path);

    // let name = 'default';
    let mut name = "default".to_string();

    // for (const attribute of node.attributes) { ... }
    for attr in &node.attributes {
        match attr {
            AttributeNode::Attribute(a) => {
                if a.name == "name" {
                    // Check if it's a text attribute (static value)
                    // A text attribute is Sequence with a single Text item
                    let is_static = matches!(
                        &a.value,
                        AttributeValue::Sequence(seq) if seq.len() == 1 && matches!(&seq[0], AttributeSequenceValue::Text(_))
                    );

                    if !is_static {
                        state
                            .analysis
                            .error(errors::slot_element_invalid_name(a.span.into()));
                    } else if let AttributeValue::Sequence(seq) = &a.value {
                        if let Some(AttributeSequenceValue::Text(text)) = seq.first() {
                            // name = attribute.value[0].data;
                            name = text.data.to_string();

                            // if (name === 'default') {
                            //     e.slot_element_invalid_name_default(attribute);
                            // }
                            if name == "default" {
                                state
                                    .analysis
                                    .error(errors::slot_element_invalid_name_default(a.span.into()));
                            }
                        }
                    }
                }
            }
            AttributeNode::SpreadAttribute(_) | AttributeNode::LetDirective(_) => {
                // These are allowed
            }
            _ => {
                // e.slot_element_invalid_attribute(attribute);
                let span = match attr {
                    AttributeNode::BindDirective(d) => d.span,
                    AttributeNode::OnDirective(d) => d.span,
                    AttributeNode::ClassDirective(d) => d.span,
                    AttributeNode::StyleDirective(d) => d.span,
                    AttributeNode::UseDirective(d) => d.span,
                    AttributeNode::TransitionDirective(d) => d.span,
                    AttributeNode::AnimateDirective(d) => d.span,
                    _ => node.span,
                };
                state
                    .analysis
                    .error(errors::slot_element_invalid_attribute(span.into()));
            }
        }
    }

    // context.state.analysis.slot_names.set(name, node);
    state
        .analysis
        .slot_names
        .insert(name, node.span.into());
}
