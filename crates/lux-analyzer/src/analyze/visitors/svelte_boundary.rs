//! SvelteBoundary visitor for analysis.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/SvelteBoundary.js`

use lux_ast::attributes::AttributeValue;
use lux_ast::elements::SvelteBoundary;
use lux_ast::node::AttributeNode;

use crate::analyze::errors;
use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;
use crate::analyze::visitors::shared::mark_subtree_dynamic;

/// Valid attribute names for svelte:boundary
const VALID_ATTRIBUTES: [&str; 3] = ["onerror", "failed", "pending"];

/// SvelteBoundary visitor.
///
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/SvelteBoundary.js`
pub fn visit_svelte_boundary(
    node: &SvelteBoundary<'_>,
    state: &mut AnalysisState<'_, '_>,
    path: &[NodeKind<'_>],
) {
    for attr in &node.attributes {
        // Only Attribute type with valid names are allowed
        match attr {
            AttributeNode::Attribute(a) => {
                if !VALID_ATTRIBUTES.contains(&a.name) {
                    state
                        .analysis
                        .error(errors::svelte_boundary_invalid_attribute(a.span.into()));
                    continue;
                }

                // Value must be an expression (not true or multiple values)
                match &a.value {
                    AttributeValue::True => {
                        state.analysis.error(errors::svelte_boundary_invalid_attribute_value(
                            a.span.into(),
                        ));
                    }
                    AttributeValue::Sequence(seq) => {
                        // Must be exactly one ExpressionTag
                        if seq.len() != 1 {
                            state.analysis.error(
                                errors::svelte_boundary_invalid_attribute_value(a.span.into()),
                            );
                        } else if !matches!(
                            &seq[0],
                            lux_ast::attributes::AttributeSequenceValue::ExpressionTag(_)
                        ) {
                            state.analysis.error(
                                errors::svelte_boundary_invalid_attribute_value(a.span.into()),
                            );
                        }
                    }
                    AttributeValue::ExpressionTag(_) => {
                        // This is valid
                    }
                }
            }
            _ => {
                let span = match attr {
                    AttributeNode::SpreadAttribute(s) => s.span,
                    AttributeNode::OnDirective(o) => o.span,
                    AttributeNode::BindDirective(b) => b.span,
                    AttributeNode::ClassDirective(c) => c.span,
                    AttributeNode::StyleDirective(s) => s.span,
                    AttributeNode::UseDirective(u) => u.span,
                    AttributeNode::TransitionDirective(t) => t.span,
                    AttributeNode::AnimateDirective(a) => a.span,
                    AttributeNode::LetDirective(l) => l.span,
                    _ => continue,
                };
                state
                    .analysis
                    .error(errors::svelte_boundary_invalid_attribute(span.into()));
            }
        }
    }

    mark_subtree_dynamic(state.analysis, path);
}
