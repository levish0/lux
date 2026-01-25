//! Element validation utilities.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/shared/element.js`

use lux_ast::attributes::{
    AnimateDirective, AttributeValue, OnDirective, TransitionDirective,
};
use lux_ast::node::AttributeNode;

use crate::analyze::errors;
use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;
use crate::analyze::warnings;

use super::attribute::{get_react_attribute_correction, validate_attribute_name};

/// Event handler modifiers.
pub const EVENT_MODIFIERS: &[&str] = &[
    "preventDefault",
    "stopPropagation",
    "stopImmediatePropagation",
    "capture",
    "once",
    "passive",
    "nonpassive",
    "self",
    "trusted",
];

/// Validates an element's attributes (RegularElement or SvelteElement).
pub fn validate_element<'a>(
    attributes: &'a [AttributeNode<'a>],
    state: &mut AnalysisState<'_, '_>,
    path: &[NodeKind<'_>],
) {
    let mut has_animate_directive = false;
    let mut in_transition: Option<&'a TransitionDirective<'a>> = None;
    let mut out_transition: Option<&'a TransitionDirective<'a>> = None;

    for attribute in attributes {
        match attribute {
            AttributeNode::Attribute(attr) => {
                validate_attribute_in_element(attr, state, path);
            }
            AttributeNode::AnimateDirective(animate) => {
                validate_animate_directive(animate, state, path, &mut has_animate_directive);
            }
            AttributeNode::TransitionDirective(transition) => {
                validate_transition_directive(
                    transition,
                    state,
                    &mut in_transition,
                    &mut out_transition,
                );
            }
            AttributeNode::OnDirective(on) => {
                validate_on_directive_modifiers(on, state);
            }
            _ => {}
        }
    }
}

/// Validates an attribute within an element.
fn validate_attribute_in_element(
    attr: &lux_ast::attributes::Attribute<'_>,
    state: &mut AnalysisState<'_, '_>,
    path: &[NodeKind<'_>],
) {
    let parent = path.last();

    // Check for event attributes (onclick, onsubmit, etc.)
    if attr.name.starts_with("on") && attr.name.len() > 2 {
        // Event attributes must have a value (not boolean attribute like `onclick`)
        if matches!(attr.value, AttributeValue::True) {
            state
                .analysis
                .error(errors::attribute_invalid_event_handler(attr.span.into()));
        }

        // Track event attributes usage
        if let Some(p) = parent {
            if matches!(p, NodeKind::RegularElement(_) | NodeKind::SvelteElement) {
                state.analysis.uses_event_attributes = true;
            }
        }
    }

    // Check for `is` attribute
    if attr.name == "is" {
        state
            .analysis
            .warning(warnings::attribute_avoid_is(attr.span.into()));
    }

    // Check for React-style attributes
    if let Some(correct) = get_react_attribute_correction(attr.name) {
        state.analysis.warning(warnings::attribute_invalid_property_name(
            attr.span.into(),
            attr.name,
            correct,
        ));
    }

    // Check for illegal colon in attribute name
    validate_attribute_name(attr, state);
}

/// Validates an animate directive.
fn validate_animate_directive(
    animate: &AnimateDirective<'_>,
    state: &mut AnalysisState<'_, '_>,
    path: &[NodeKind<'_>],
    has_animate: &mut bool,
) {
    // Check parent - must be in an EachBlock
    let grandparent = path.get(path.len().saturating_sub(2));

    if !matches!(grandparent, Some(NodeKind::EachBlock)) {
        state
            .analysis
            .error(errors::animation_invalid_placement(animate.span.into()));
    }

    // Check for duplicate animate directive
    if *has_animate {
        state
            .analysis
            .error(errors::animation_duplicate(animate.span.into()));
    } else {
        *has_animate = true;
    }
}

/// Validates a transition directive.
fn validate_transition_directive<'a>(
    transition: &'a TransitionDirective<'a>,
    state: &mut AnalysisState<'_, '_>,
    in_transition: &mut Option<&'a TransitionDirective<'a>>,
    out_transition: &mut Option<&'a TransitionDirective<'a>>,
) {
    let existing = if transition.intro && in_transition.is_some() {
        *in_transition
    } else if transition.outro && out_transition.is_some() {
        *out_transition
    } else {
        None
    };

    if let Some(existing) = existing {
        let a = if existing.intro {
            if existing.outro {
                "transition"
            } else {
                "in"
            }
        } else {
            "out"
        };

        let b = if transition.intro {
            if transition.outro {
                "transition"
            } else {
                "in"
            }
        } else {
            "out"
        };

        if a == b {
            state
                .analysis
                .error(errors::transition_duplicate(transition.span.into(), a));
        } else {
            state
                .analysis
                .error(errors::transition_conflict(transition.span.into(), a, b));
        }
    }

    if transition.intro {
        *in_transition = Some(transition);
    }
    if transition.outro {
        *out_transition = Some(transition);
    }
}

/// Validates event handler modifiers.
fn validate_on_directive_modifiers(on: &OnDirective<'_>, state: &mut AnalysisState<'_, '_>) {
    let mut has_passive = false;
    let mut conflicting_passive: Option<&str> = None;

    for &modifier in &on.modifiers {
        if !EVENT_MODIFIERS.contains(&modifier) {
            let list = format!(
                "{} or {}",
                EVENT_MODIFIERS[..EVENT_MODIFIERS.len() - 1].join(", "),
                EVENT_MODIFIERS.last().unwrap()
            );
            state
                .analysis
                .error(errors::event_handler_invalid_modifier(on.span.into(), &list));
        }

        if modifier == "passive" {
            has_passive = true;
        } else if modifier == "nonpassive" || modifier == "preventDefault" {
            conflicting_passive = Some(modifier);
        }

        if has_passive && conflicting_passive.is_some() {
            state.analysis.error(
                errors::event_handler_invalid_modifier_combination(
                    on.span.into(),
                    "passive",
                    conflicting_passive.unwrap(),
                ),
            );
        }
    }
}
