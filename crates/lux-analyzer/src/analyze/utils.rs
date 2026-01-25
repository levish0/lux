//! Analysis utility functions.

use oxc_ast::ast::{BindingPattern, CallExpression, Expression};

use crate::scope::ScopeTree;

/// Returns the name of the rune if the given expression is a `CallExpression` using a rune.
pub fn get_rune<'a>(expr: &'a Expression<'a>, scope_tree: &ScopeTree) -> Option<&'static str> {
    let call = match expr {
        Expression::CallExpression(call) => call.as_ref(),
        _ => return None,
    };

    get_rune_from_call(call, scope_tree)
}

/// Returns the name of the rune if the given call expression is using a rune.
pub fn get_rune_from_call<'a>(
    call: &'a CallExpression<'a>,
    scope_tree: &ScopeTree,
) -> Option<&'static str> {
    let keypath = get_global_keypath(&call.callee, scope_tree)?;

    if is_rune(&keypath) {
        Some(match keypath.as_str() {
            "$state" => "$state",
            "$state.raw" => "$state.raw",
            "$state.snapshot" => "$state.snapshot",
            "$derived" => "$derived",
            "$derived.by" => "$derived.by",
            "$props" => "$props",
            "$props.id" => "$props.id",
            "$bindable" => "$bindable",
            "$effect" => "$effect",
            "$effect.pre" => "$effect.pre",
            "$effect.root" => "$effect.root",
            "$effect.tracking" => "$effect.tracking",
            "$inspect" => "$inspect",
            "$inspect.trace" => "$inspect.trace",
            "$host" => "$host",
            _ => return None,
        })
    } else {
        None
    }
}

/// Gets the global keypath from an expression.
/// Returns something like "$state", "$state.raw", "$derived.by", etc.
fn get_global_keypath(expr: &Expression<'_>, scope_tree: &ScopeTree) -> Option<String> {
    let mut n = expr;
    let mut joined = String::new();

    // Handle member expressions like `$state.raw`
    while let Expression::StaticMemberExpression(member) = n {
        joined = format!(".{}{}", member.property.name, joined);
        n = &member.object;
    }

    // Handle call expressions like `$derived.by()`
    if let Expression::CallExpression(call) = n {
        if let Expression::Identifier(id) = &call.callee {
            joined = format!("(){}", joined);
            // Check if the identifier is not bound (i.e., it's a global/rune)
            // TODO: Check scope properly once scope tree is fully integrated
            let _ = scope_tree; // Suppress unused warning for now
            return Some(format!("{}{}", id.name, joined));
        }
    }

    // Handle simple identifiers
    if let Expression::Identifier(id) = n {
        // TODO: Check scope properly once scope tree is fully integrated
        return Some(format!("{}{}", id.name, joined));
    }

    None
}

/// Returns true if the given name is a rune.
pub fn is_rune(name: &str) -> bool {
    matches!(
        name,
        "$state"
            | "$state.raw"
            | "$state.snapshot"
            | "$derived"
            | "$derived.by"
            | "$props"
            | "$props.id"
            | "$bindable"
            | "$effect"
            | "$effect.pre"
            | "$effect.root"
            | "$effect.tracking"
            | "$inspect"
            | "$inspect.trace"
            | "$inspect.with"
            | "$host"
    )
}

/// Extract all identifiers from a destructuring pattern.
pub fn extract_identifiers_from_pattern<'a>(
    pattern: &'a BindingPattern<'a>,
) -> Vec<&'a oxc_ast::ast::BindingIdentifier<'a>> {
    let mut ids = Vec::new();
    extract_identifiers_recursive(pattern, &mut ids);
    ids
}

fn extract_identifiers_recursive<'a>(
    pattern: &'a BindingPattern<'a>,
    ids: &mut Vec<&'a oxc_ast::ast::BindingIdentifier<'a>>,
) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => {
            ids.push(id);
        }
        BindingPattern::ObjectPattern(obj) => {
            for prop in &obj.properties {
                extract_identifiers_recursive(&prop.value, ids);
            }
            if let Some(rest) = &obj.rest {
                extract_identifiers_recursive(&rest.argument, ids);
            }
        }
        BindingPattern::ArrayPattern(arr) => {
            for elem in arr.elements.iter().flatten() {
                extract_identifiers_recursive(elem, ids);
            }
            if let Some(rest) = &arr.rest {
                extract_identifiers_recursive(&rest.argument, ids);
            }
        }
        BindingPattern::AssignmentPattern(assign) => {
            extract_identifiers_recursive(&assign.left, ids);
        }
    }
}

use crate::scope::{Binding, DeclarationKind};

/// Validates that a variable name doesn't start with $ unless it's allowed.
/// Returns an error type if invalid.
pub enum IdentifierNameError {
    /// The name is exactly "$"
    DollarBinding,
    /// The name starts with "$" but is not allowed
    DollarPrefix,
}

/// Validates that an identifier name is valid (not $ prefixed).
/// The function_depth argument is for backwards compatibility - in Svelte 4
/// you were allowed to define $-prefixed variables below the top level.
pub fn validate_identifier_name(
    binding: &Binding,
    function_depth: Option<usize>,
) -> Option<IdentifierNameError> {
    let declaration_kind = binding.declaration_kind;

    // Params and synthetic bindings can have $ prefix
    if declaration_kind == DeclarationKind::Param
        || declaration_kind == DeclarationKind::RestParam
        || declaration_kind == DeclarationKind::Synthetic
    {
        return None;
    }

    // In legacy mode, $ prefix is allowed below the top level
    if let Some(depth) = function_depth {
        if depth > 1 {
            return None;
        }
    }

    let name = &binding.name;

    if name == "$" {
        return Some(IdentifierNameError::DollarBinding);
    }

    if name.starts_with('$') {
        return Some(IdentifierNameError::DollarPrefix);
    }

    None
}

/// Checks if an expression is a "safe" identifier - one that doesn't
/// require component context to exist (e.g., not an import or prop).
pub fn is_safe_identifier(expr: &Expression<'_>, scope_tree: &ScopeTree, scope_id: crate::scope::ScopeId) -> bool {
    let mut n = expr;

    // Walk through member expressions to get the base identifier
    while let Expression::StaticMemberExpression(member) = n {
        n = &member.object;
    }
    while let Expression::ComputedMemberExpression(member) = n {
        n = &member.object;
    }

    match n {
        Expression::Identifier(id) => {
            let name = id.name.as_str();

            // Look up the binding from the given scope
            let binding = scope_tree.get(scope_id, name);

            if binding.is_none() {
                return true; // Globals are assumed safe
            }

            if let Some(binding_id) = binding {
                let binding = scope_tree.get_binding(binding_id);

                // Store subscriptions need to check the underlying store
                if binding.kind == crate::scope::BindingKind::StoreSub {
                    // Would need to recursively check the store, but for now return true
                    return true;
                }

                // These binding kinds require component context
                use crate::scope::BindingKind;
                !matches!(
                    binding.kind,
                    BindingKind::Prop | BindingKind::BindableProp | BindingKind::RestProp
                ) && binding.declaration_kind != DeclarationKind::Import
            } else {
                true
            }
        }
        _ => false,
    }
}

/// Checks if an expression is "pure" - meaning it doesn't have side effects
/// and doesn't depend on reactive state.
pub fn is_pure(expr: &Expression<'_>, scope_tree: &ScopeTree, scope_id: crate::scope::ScopeId) -> bool {
    match expr {
        Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::RegExpLiteral(_)
        | Expression::StringLiteral(_) => true,

        Expression::Identifier(id) => {
            let name = id.name.as_str();
            // Check if it's bound
            let binding = scope_tree.get(scope_id, name);
            // Unbound identifiers (globals) are assumed pure
            binding.is_none()
        }

        Expression::StaticMemberExpression(member) => is_pure(&member.object, scope_tree, scope_id),
        Expression::ComputedMemberExpression(member) => {
            is_pure(&member.object, scope_tree, scope_id)
                && is_pure(&member.expression, scope_tree, scope_id)
        }

        Expression::CallExpression(call) => {
            // Check if the callee is pure
            if !is_pure(&call.callee, scope_tree, scope_id) {
                return false;
            }
            // Check all arguments
            for arg in &call.arguments {
                match arg {
                    oxc_ast::ast::Argument::SpreadElement(spread) => {
                        if !is_pure(&spread.argument, scope_tree, scope_id) {
                            return false;
                        }
                    }
                    _ => {
                        if !is_pure(arg.to_expression(), scope_tree, scope_id) {
                            return false;
                        }
                    }
                }
            }
            true
        }

        _ => false,
    }
}
