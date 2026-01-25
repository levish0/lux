//! Component visitor for analysis.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/Component.js`

use lux_ast::elements::Component;
use lux_ast::node::AttributeNode;

use crate::analyze::analysis::ComponentMeta;
use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;
use crate::analyze::visitors::shared::mark_subtree_dynamic;

/// Component visitor.
///
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/Component.js`
pub fn visit_component(
    node: &Component<'_>,
    state: &mut AnalysisState<'_, '_>,
    path: &[NodeKind<'_>],
) {
    // const binding = context.state.scope.get(
    //     node.name.includes('.') ? node.name.slice(0, node.name.indexOf('.')) : node.name
    // );
    let base_name = if let Some(dot_pos) = node.name.find('.') {
        &node.name[..dot_pos]
    } else {
        node.name
    };

    // node.metadata.dynamic = context.state.analysis.runes &&
    //     binding !== null && (binding.kind !== 'normal' || node.name.includes('.'));
    // For now, we can't fully implement this without scope resolution
    // TODO: Implement proper dynamic detection when scope is available
    let _has_dot = node.name.contains('.');

    // node.metadata.has_spread
    let has_spread = node.attributes.iter().any(|attr| {
        matches!(attr, AttributeNode::SpreadAttribute(_))
    });

    // Store metadata
    let meta = state
        .analysis
        .component_meta
        .entry(node.span.into())
        .or_insert_with(ComponentMeta::default);
    meta.has_spread = has_spread;

    // visit_component (shared) does a lot of work:
    // - Validates attributes (only Attribute, SpreadAttribute, LetDirective, OnDirective, BindDirective allowed)
    // - Validates OnDirective modifiers (only 'once' allowed)
    // - Tracks uses_component_bindings for bind:xxx (where xxx !== 'this')
    // - Links snippets to component

    // Check for bind:xxx directives (not this)
    for attr in &node.attributes {
        if let AttributeNode::BindDirective(bind) = attr {
            if bind.name != "this" {
                state.analysis.uses_component_bindings = true;
            }
        }
    }

    // mark_subtree_dynamic(context.path);
    mark_subtree_dynamic(state.analysis, path);

    // Note: The scope lookup for `base_name` would be done here to check if it's
    // an imported component and track expression metadata. This requires scope resolution.
    let _ = base_name; // Suppress unused warning
}
