//! Attribute visitor.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/Attribute.js`

use lux_ast::attributes::{Attribute, AttributeValue};
use oxc_ast::ast::Expression;

use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;
use crate::analyze::visitors::shared::mark_subtree_dynamic;

/// Events that can be delegated to the document root.
const DELEGATED_EVENTS: &[&str] = &[
    "beforeinput",
    "click",
    "change",
    "dblclick",
    "contextmenu",
    "focusin",
    "focusout",
    "input",
    "keydown",
    "keyup",
    "mousedown",
    "mousemove",
    "mouseout",
    "mouseover",
    "mouseup",
    "pointerdown",
    "pointermove",
    "pointerout",
    "pointerover",
    "pointerup",
    "touchend",
    "touchmove",
    "touchstart",
];

/// Properties that cannot be set statically in the template.
const NON_STATIC_PROPERTIES: &[&str] = &["autofocus", "muted", "defaultValue", "defaultChecked"];

/// Returns true if the event can be delegated.
pub fn can_delegate_event(event_name: &str) -> bool {
    DELEGATED_EVENTS.contains(&event_name)
}

/// Returns true if the attribute cannot be set statically.
pub fn cannot_be_set_statically(name: &str) -> bool {
    NON_STATIC_PROPERTIES.contains(&name)
}

/// Returns true if the attribute is an event attribute (starts with "on").
pub fn is_event_attribute(attr: &Attribute<'_>) -> bool {
    // Event attributes have a single expression value and name starts with "on"
    if !attr.name.starts_with("on") {
        return false;
    }

    // Must be an expression attribute (single expression, not text chunks)
    matches!(&attr.value, AttributeValue::ExpressionTag(_))
}

/// Visits an Attribute node.
pub fn visit_attribute<'a>(
    node: &Attribute<'a>,
    state: &mut AnalysisState<'_, '_>,
    path: &[NodeKind<'a>],
) {
    // Get parent element
    let parent = path.last();

    // Special case: <option value="" />
    if let Some(NodeKind::RegularElement(tag_name)) = parent {
        if node.name == "value" && *tag_name == "option" {
            mark_subtree_dynamic(state.analysis, path);
        }
    }

    // Event attributes need dynamic handling
    if is_event_attribute(node) {
        mark_subtree_dynamic(state.analysis, path);

        // Track event attribute usage
        if matches!(parent, Some(NodeKind::RegularElement(_)) | Some(NodeKind::SvelteElement)) {
            state.analysis.uses_event_attributes = true;
        }
    }

    // Attributes that cannot be set statically
    if cannot_be_set_statically(&node.name) {
        mark_subtree_dynamic(state.analysis, path);
    }

    // class={expression} that's not a simple literal needs clsx
    if node.name == "class" {
        if let AttributeValue::ExpressionTag(expr_tag) = &node.value {
            let needs_clsx = !matches!(
                &expr_tag.expression,
                Expression::StringLiteral(_)
                    | Expression::TemplateLiteral(_)
                    | Expression::BinaryExpression(_)
            );

            if needs_clsx {
                mark_subtree_dynamic(state.analysis, path);
                // TODO: Set metadata.needs_clsx when we have attribute metadata
            }
        }
    }
}
