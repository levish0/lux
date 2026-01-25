//! Identifier visitor for analysis.

use lux_utils::{get_rune, is_rune, Rune};
use oxc_ast::ast::IdentifierReference;

use crate::analyze::errors;
use crate::analyze::state::AnalysisState;

/// Visits an identifier reference during analysis.
pub fn visit_identifier(node: &IdentifierReference<'_>, state: &mut AnalysisState<'_, '_>) {
    let name = node.name.as_str();

    // Check for $$slots usage
    if name == "$$slots" {
        state.analysis.uses_slots = true;
    }

    // Check for $$props usage (legacy mode)
    if !state.analysis.runes && name == "$$props" {
        state.analysis.uses_props = true;
    }

    // Check for $$restProps usage (legacy mode)
    if !state.analysis.runes && name == "$$restProps" {
        state.analysis.uses_rest_props = true;
    }

    // Check for `arguments` usage outside of functions
    if name == "arguments" && !state.in_function() {
        state
            .analysis
            .error(errors::invalid_arguments_usage(node.span));
    }

    // In runes mode, validate rune references
    if state.analysis.runes && is_rune(name) {
        // Check if this rune is not shadowed by a local binding
        if state.analysis.scope_tree.get(state.scope, name).is_none() {
            // A rune identifier not followed by () is an error
            // This is detected in CallExpression visitor by checking parent context
        }
    }

    // Look up the binding
    if let Some(binding_id) = state.analysis.scope_tree.get(state.scope, name) {
        let _binding = state.analysis.scope_tree.get_binding(binding_id);

        // Track dependencies in reactive contexts
        // TODO: Add expression metadata tracking
    }
}

/// Gets the deprecated or renamed rune info for error messages.
pub fn get_deprecated_rune_info(base: &str, property: &str) -> Option<DeprecatedRuneInfo> {
    match (base, property) {
        ("$state", "frozen") => Some(DeprecatedRuneInfo::Renamed {
            old: "$state.frozen",
            new: Rune::StateRaw,
        }),
        ("$effect", "active") => Some(DeprecatedRuneInfo::Renamed {
            old: "$effect.active",
            new: Rune::EffectTracking,
        }),
        ("$state", "is") => Some(DeprecatedRuneInfo::Removed { name: "$state.is" }),
        _ => None,
    }
}

/// Information about a deprecated rune.
pub enum DeprecatedRuneInfo {
    /// The rune was renamed to a new name.
    Renamed { old: &'static str, new: Rune },
    /// The rune was removed entirely.
    Removed { name: &'static str },
}

/// Validates a rune member access chain.
/// Returns the rune if valid, or None if invalid.
pub fn validate_rune_member_access(base: &str, property: &str) -> Option<Rune> {
    let full_name = format!("{}.{}", base, property);
    get_rune(&full_name)
}
