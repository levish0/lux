//! Attribute validation utilities.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/shared/attribute.js`

use lux_ast::attributes::Attribute;

use crate::analyze::state::AnalysisState;
use crate::analyze::warnings;

/// Validates an attribute name for illegal colons.
pub fn validate_attribute_name(attr: &Attribute<'_>, state: &mut AnalysisState<'_, '_>) {
    if attr.name.contains(':')
        && !attr.name.starts_with("xmlns:")
        && !attr.name.starts_with("xlink:")
        && !attr.name.starts_with("xml:")
    {
        state
            .analysis
            .warning(warnings::attribute_illegal_colon(attr.span.into()));
    }
}

/// React-style attribute name corrections.
const REACT_ATTRIBUTES: &[(&str, &str)] = &[("className", "class"), ("htmlFor", "for")];

/// Checks if a name is a React-style attribute and returns the correct name.
pub fn get_react_attribute_correction(name: &str) -> Option<&'static str> {
    REACT_ATTRIBUTES
        .iter()
        .find(|(react, _)| *react == name)
        .map(|(_, correct)| *correct)
}
