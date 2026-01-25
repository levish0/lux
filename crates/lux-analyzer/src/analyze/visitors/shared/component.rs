//! Component validation utilities.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/shared/component.js`

use lux_ast::node::AttributeNode;

use crate::analyze::errors;
use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;
use crate::analyze::visitors::shared::mark_subtree_dynamic;

/// Shared component validation for Component, SvelteComponent, SvelteSelf.
///
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/shared/component.js`
pub fn visit_component_like(
    attributes: &[AttributeNode<'_>],
    state: &mut AnalysisState<'_, '_>,
    path: &[NodeKind<'_>],
) {
    // Check for invalid directives on components
    for attr in attributes {
        match attr {
            AttributeNode::OnDirective(on) => {
                // On components, only the `once` modifier is allowed
                for modifier in &on.modifiers {
                    if *modifier != "once" {
                        state.analysis.error(errors::event_handler_invalid_component_modifier(
                            on.span.into(),
                        ));
                        break;
                    }
                }
            }
            AttributeNode::AnimateDirective(anim) => {
                state
                    .analysis
                    .error(errors::component_invalid_directive(anim.span.into()));
            }
            AttributeNode::TransitionDirective(trans) => {
                state
                    .analysis
                    .error(errors::component_invalid_directive(trans.span.into()));
            }
            AttributeNode::UseDirective(use_dir) => {
                state
                    .analysis
                    .error(errors::component_invalid_directive(use_dir.span.into()));
            }
            AttributeNode::StyleDirective(style) => {
                state
                    .analysis
                    .error(errors::component_invalid_directive(style.span.into()));
            }
            AttributeNode::ClassDirective(class) => {
                state
                    .analysis
                    .error(errors::component_invalid_directive(class.span.into()));
            }
            AttributeNode::BindDirective(bind) => {
                // bind:this is allowed, but other bindings set uses_component_bindings
                if bind.name != "this" {
                    state.analysis.uses_component_bindings = true;
                }
            }
            _ => {}
        }
    }

    mark_subtree_dynamic(state.analysis, path);
}
