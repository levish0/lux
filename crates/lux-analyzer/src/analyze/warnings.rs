//! Analysis warnings.
//!
//! These are compile-time warnings detected during semantic analysis.

use oxc_span::Span;

/// An analysis warning.
#[derive(Debug, Clone)]
pub struct AnalysisWarning {
    pub code: WarningCode,
    pub message: String,
    pub span: Span,
}

impl AnalysisWarning {
    pub fn new(code: WarningCode, message: impl Into<String>, span: Span) -> Self {
        Self {
            code,
            message: message.into(),
            span,
        }
    }
}

/// Warning codes for analysis warnings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningCode {
    // Event warnings
    EventDirectiveDeprecated,

    // Attribute warnings
    AttributeAvoidIs,
    AttributeInvalidPropertyName,
    AttributeGlobalEventReference,
    AttributeIllegalColon,
    AttributeQuoted,

    // Element warnings
    ElementInvalidSelfClosingTag,
    ComponentNameLowercase,

    // Placement warnings
    NodeInvalidPlacementSsr,

    // Block warnings
    BlockEmpty,
}

// =============================================================================
// Warning constructors
// =============================================================================

/// `on:` directive is deprecated in runes mode, use `onclick` instead
pub fn event_directive_deprecated(span: Span, name: &str) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::EventDirectiveDeprecated,
        format!(
            "Using `on:{}` is deprecated. Use the `on{}` attribute instead.",
            name, name
        ),
        span,
    )
}

/// Avoid using the `is` attribute
pub fn attribute_avoid_is(span: Span) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::AttributeAvoidIs,
        "The `is` attribute is not supported cross-browser and should be avoided".to_string(),
        span,
    )
}

/// React-style attribute name (className -> class)
pub fn attribute_invalid_property_name(
    span: Span,
    wrong: &str,
    correct: &str,
) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::AttributeInvalidPropertyName,
        format!("`{}` is not a valid attribute name. Did you mean `{}`?", wrong, correct),
        span,
    )
}

/// Referencing global event handler
pub fn attribute_global_event_reference(span: Span, name: &str) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::AttributeGlobalEventReference,
        format!(
            "`{}` refers to a global event handler. Did you forget to declare a variable?",
            name
        ),
        span,
    )
}

/// Self-closing tag on non-void element
pub fn element_invalid_self_closing_tag(span: Span, name: &str) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::ElementInvalidSelfClosingTag,
        format!(
            "`<{}/>` is a self-closing tag, but `<{}>` is not a void element. Use `<{}></{}>` instead.",
            name, name, name, name
        ),
        span,
    )
}

/// Component name looks like element (lowercase)
pub fn component_name_lowercase(span: Span, name: &str) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::ComponentNameLowercase,
        format!(
            "`<{}>` looks like a component, but component names must start with a capital letter. \
             If this is a custom element, add it to the `customElement` option.",
            name
        ),
        span,
    )
}

/// Attribute name contains illegal colon
pub fn attribute_illegal_colon(span: Span) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::AttributeIllegalColon,
        "Attributes should not contain ':' characters to prevent ambiguity with Svelte directives",
        span,
    )
}

/// Unnecessary quotes around attribute value
pub fn attribute_quoted(span: Span) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::AttributeQuoted,
        "Attribute value is unnecessarily quoted",
        span,
    )
}

/// Element placement warning for SSR
pub fn node_invalid_placement_ssr(span: Span, message: &str) -> AnalysisWarning {
    AnalysisWarning::new(WarningCode::NodeInvalidPlacementSsr, message.to_string(), span)
}

/// Block body is empty (contains only whitespace)
pub fn block_empty(span: Span) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::BlockEmpty,
        "Empty block",
        span,
    )
}
