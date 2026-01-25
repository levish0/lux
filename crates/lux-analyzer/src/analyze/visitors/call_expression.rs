//! CallExpression visitor for analysis.
//!
//! Handles rune calls ($state, $derived, $effect, etc.) and other function calls.

use lux_utils::{get_rune, is_rune, Rune};
use oxc_ast::ast::{Argument, CallExpression, Expression};

use crate::analyze::errors;
use crate::analyze::state::{AnalysisState, AstType};

/// Gets the rune from a call expression, if it's a rune call.
pub fn get_rune_from_call(node: &CallExpression<'_>, state: &AnalysisState<'_, '_>) -> Option<Rune> {
    match &node.callee {
        Expression::Identifier(id) => {
            let name = id.name.as_str();
            if is_rune(name) && state.analysis.scope_tree.get(state.scope, name).is_none() {
                get_rune(name)
            } else {
                None
            }
        }
        Expression::StaticMemberExpression(member) => {
            // Handle $state.raw, $derived.by, etc.
            if let Expression::Identifier(obj) = &member.object {
                let base = obj.name.as_str();
                if base.starts_with('$') {
                    let full_name = format!("{}.{}", base, member.property.name);
                    if state.analysis.scope_tree.get(state.scope, base).is_none() {
                        return get_rune(&full_name);
                    }
                }
            }
            None
        }
        Expression::CallExpression(inner) => {
            // Handle $inspect().with
            if let Some(Rune::Inspect) = get_rune_from_call(inner, state) {
                if let Expression::StaticMemberExpression(member) = &node.callee {
                    if member.property.name.as_str() == "with" {
                        return Some(Rune::InspectWith);
                    }
                }
            }
            None
        }
        _ => None,
    }
}

/// Checks if any argument is a spread element.
fn has_spread_argument(args: &[Argument<'_>]) -> bool {
    args.iter()
        .any(|arg| matches!(arg, Argument::SpreadElement(_)))
}

/// Visits a call expression during analysis.
pub fn visit_call_expression(node: &CallExpression<'_>, state: &mut AnalysisState<'_, '_>) {
    let rune = get_rune_from_call(node, state);

    // Validate no spread arguments for runes (except $inspect)
    if let Some(r) = rune {
        if r != Rune::Inspect && has_spread_argument(&node.arguments) {
            state
                .analysis
                .error(errors::rune_invalid_spread(node.span, r.as_str()));
        }
    }

    match rune {
        Some(Rune::Props) => {
            if state.has_props_rune {
                state
                    .analysis
                    .error(errors::props_duplicate(node.span, "$props"));
            }
            state.has_props_rune = true;

            // Validation: must be in instance script at top level
            if state.ast_type != AstType::Instance {
                state
                    .analysis
                    .error(errors::props_invalid_placement(node.span));
            }

            if !node.arguments.is_empty() {
                state
                    .analysis
                    .error(errors::rune_invalid_arguments(node.span, "$props"));
            }
        }

        Some(Rune::PropsId) => {
            // Validation: must be in instance script at top level
            if state.ast_type != AstType::Instance {
                state
                    .analysis
                    .error(errors::props_invalid_placement(node.span));
            }

            if !node.arguments.is_empty() {
                state
                    .analysis
                    .error(errors::rune_invalid_arguments(node.span, "$props.id"));
            }
        }

        Some(r @ Rune::State) | Some(r @ Rune::StateRaw) => {
            // Validation: must be in valid declaration context
            // Note: Full validation requires parent context
            if node.arguments.len() > 1 {
                state.analysis.error(errors::rune_invalid_arguments_length(
                    node.span,
                    r.as_str(),
                    "zero or one arguments",
                ));
            }
        }

        Some(Rune::Derived) => {
            if node.arguments.len() != 1 {
                state.analysis.error(errors::rune_invalid_arguments_length(
                    node.span,
                    "$derived",
                    "exactly one argument",
                ));
            }
        }

        Some(Rune::DerivedBy) => {
            if node.arguments.len() != 1 {
                state.analysis.error(errors::rune_invalid_arguments_length(
                    node.span,
                    "$derived.by",
                    "exactly one argument",
                ));
            }
        }

        Some(r @ Rune::Effect) | Some(r @ Rune::EffectPre) => {
            // Validation: must be an expression statement (requires parent context)
            if node.arguments.len() != 1 {
                state.analysis.error(errors::rune_invalid_arguments_length(
                    node.span,
                    r.as_str(),
                    "exactly one argument",
                ));
            }
            state.analysis.needs_context = true;
        }

        Some(Rune::EffectTracking) => {
            if !node.arguments.is_empty() {
                state
                    .analysis
                    .error(errors::rune_invalid_arguments(node.span, "$effect.tracking"));
            }
        }

        Some(Rune::EffectRoot) => {
            if node.arguments.len() != 1 {
                state.analysis.error(errors::rune_invalid_arguments_length(
                    node.span,
                    "$effect.root",
                    "exactly one argument",
                ));
            }
        }

        Some(Rune::EffectPending) => {
            // Mark as having state (expression metadata)
        }

        Some(Rune::Bindable) => {
            if node.arguments.len() > 1 {
                state.analysis.error(errors::rune_invalid_arguments_length(
                    node.span,
                    "$bindable",
                    "zero or one arguments",
                ));
            }
            // Validation: must be in $props destructuring (requires parent context)
            state.analysis.needs_context = true;
        }

        Some(Rune::Host) => {
            if !node.arguments.is_empty() {
                state
                    .analysis
                    .error(errors::rune_invalid_arguments(node.span, "$host"));
            }
            if state.ast_type == AstType::Module || !state.analysis.custom_element {
                state
                    .analysis
                    .error(errors::host_invalid_placement(node.span));
            }
        }

        Some(Rune::Inspect) => {
            if node.arguments.is_empty() {
                state.analysis.error(errors::rune_invalid_arguments_length(
                    node.span,
                    "$inspect",
                    "one or more arguments",
                ));
            }
        }

        Some(Rune::InspectWith) => {
            if node.arguments.len() != 1 {
                state.analysis.error(errors::rune_invalid_arguments_length(
                    node.span,
                    "$inspect().with",
                    "exactly one argument",
                ));
            }
        }

        Some(Rune::InspectTrace) => {
            if node.arguments.len() > 1 {
                state.analysis.error(errors::rune_invalid_arguments_length(
                    node.span,
                    "$inspect.trace",
                    "zero or one arguments",
                ));
            }
            state.analysis.base.tracing = true;
        }

        Some(Rune::StateEager) => {
            if node.arguments.len() != 1 {
                state.analysis.error(errors::rune_invalid_arguments_length(
                    node.span,
                    "$state.eager",
                    "exactly one argument",
                ));
            }
        }

        Some(Rune::StateSnapshot) => {
            if node.arguments.len() != 1 {
                state.analysis.error(errors::rune_invalid_arguments_length(
                    node.span,
                    "$state.snapshot",
                    "exactly one argument",
                ));
            }
        }

        None => {
            // Regular function call - may need context for dynamic calls
            // TODO: Check if callee is "safe" (doesn't need context)
        }
    }
}
