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
    SlotElementDeprecated,
    SvelteComponentDeprecated,
    SvelteSelfDeprecated,

    // Placement warnings
    NodeInvalidPlacementSsr,

    // Block warnings
    BlockEmpty,

    // Reactive statement warnings
    ReactiveDeclarationInvalidPlacement,
    ReactiveDeclarationModuleScriptDependency,

    // State warnings
    StateReferencedLocally,

    // Text/security warnings
    BidirectionalControlCharacters,

    // Legacy warnings
    LegacyComponentCreation,

    // Performance warnings
    PerfAvoidInlineClass,

    // A11y warnings
    A11yAccesskey,
    A11yAriaActivedescendantHasTabindex,
    A11yAriaAttributes,
    A11yAutocompleteValid,
    A11yAutofocus,
    A11yClickEventsHaveKeyEvents,
    A11yConsiderExplicitLabel,
    A11yDistractingElements,
    A11yHidden,
    A11yImgRedundantAlt,
    A11yInteractiveSupportsFocus,
    A11yInvalidAttribute,
    A11yLabelHasAssociatedControl,
    A11yMediaHasCaption,
    A11yMisplacedRole,
    A11yMisplacedScope,
    A11yMissingAttribute,
    A11yMissingContent,
    A11yMouseEventsHaveKeyEvents,
    A11yNoAbstractRole,
    A11yNoInteractiveElementToNoninteractiveRole,
    A11yNoNoninteractiveElementInteractions,
    A11yNoNoninteractiveElementToInteractiveRole,
    A11yNoNoninteractiveTabindex,
    A11yNoRedundantRoles,
    A11yNoStaticElementInteractions,
    A11yPositiveTabindex,
    A11yRoleHasRequiredAriaProps,
    A11yRoleSupportsAriaProps,
    A11yUnknownAriaAttribute,
    A11yUnknownRole,
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

/// Slot element is deprecated in runes mode
pub fn slot_element_deprecated(span: Span) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::SlotElementDeprecated,
        "`<slot>` is deprecated in runes mode. Use snippets instead.",
        span,
    )
}

/// svelte:component is deprecated in runes mode
pub fn svelte_component_deprecated(span: Span) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::SvelteComponentDeprecated,
        "`<svelte:component>` is deprecated in runes mode. Use `<Component>` instead.",
        span,
    )
}

/// svelte:self is deprecated in runes mode
pub fn svelte_self_deprecated(span: Span, name: &str, basename: &str) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::SvelteSelfDeprecated,
        format!(
            "`<svelte:self>` is deprecated. Use `<{}>` to reference this component's constructor. \
             Import `{}` and use it instead.",
            name, basename
        ),
        span,
    )
}

/// Reactive declaration ($:) is not at the top level of the instance script
pub fn reactive_declaration_invalid_placement(span: Span) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::ReactiveDeclarationInvalidPlacement,
        "`$:` is not valid at this position. It should be at the top level of your `<script>` block",
        span,
    )
}

/// Reactive declaration depends on a module script variable that is reassigned
pub fn reactive_declaration_module_script_dependency(span: Span) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::ReactiveDeclarationModuleScriptDependency,
        "Reactive declaration references a variable declared in the module script that is reassigned. This may cause incorrect reactivity.",
        span,
    )
}

/// State is referenced in the same scope it was declared
pub fn state_referenced_locally(span: Span, name: &str, context: &str) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::StateReferencedLocally,
        format!(
            "State variable `{}` is read inside a {} without using `$state.snapshot()`. \
             This may cause stale values to be captured.",
            name, context
        ),
        span,
    )
}

/// Bidirectional control characters detected in text
pub fn bidirectional_control_characters(span: Span) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::BidirectionalControlCharacters,
        "Bidirectional control characters detected. These can be used to create security vulnerabilities in code.",
        span,
    )
}

/// Legacy component creation pattern detected
pub fn legacy_component_creation(span: Span) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::LegacyComponentCreation,
        "Svelte 5 components are no longer classes. Instantiate them using `mount` or `hydrate` instead.",
        span,
    )
}

/// Performance warning for inline class creation
pub fn perf_avoid_inline_class(span: Span) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::PerfAvoidInlineClass,
        "Avoid creating classes inline in functions. This creates a new class instance on every call, which can impact performance.",
        span,
    )
}

// =============================================================================
// A11y warning constructors
// =============================================================================

/// Avoid using accesskey
pub fn a11y_accesskey(span: Span) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::A11yAccesskey,
        "Avoid using accesskey",
        span,
    )
}

/// aria-activedescendant should be on element with tabindex
pub fn a11y_aria_activedescendant_has_tabindex(span: Span) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::A11yAriaActivedescendantHasTabindex,
        "An element with an aria-activedescendant attribute should have a tabindex value",
        span,
    )
}

/// ARIA attributes not supported on element
pub fn a11y_aria_attributes(span: Span, element: &str) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::A11yAriaAttributes,
        format!("`<{}>` should not have aria-* attributes", element),
        span,
    )
}

/// Autofocus attribute should be avoided
pub fn a11y_autofocus(span: Span) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::A11yAutofocus,
        "Avoid using autofocus",
        span,
    )
}

/// Distracting element warning
pub fn a11y_distracting_elements(span: Span, element: &str) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::A11yDistractingElements,
        format!("Avoid `<{}>` elements", element),
        span,
    )
}

/// aria-hidden on heading
pub fn a11y_hidden(span: Span, element: &str) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::A11yHidden,
        format!("`<{}>` element should not be hidden", element),
        span,
    )
}

/// Redundant alt text on image
pub fn a11y_img_redundant_alt(span: Span) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::A11yImgRedundantAlt,
        "Screenreaders already announce `<img>` elements as an image. Avoid redundant alt text like 'image', 'photo', 'picture'.",
        span,
    )
}

/// Missing attribute warning
pub fn a11y_missing_attribute(span: Span, element: &str, attribute: &str) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::A11yMissingAttribute,
        format!("`<{}>` element should have an {} attribute", element, attribute),
        span,
    )
}

/// Missing content warning
pub fn a11y_missing_content(span: Span, element: &str) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::A11yMissingContent,
        format!("`<{}>` element should have child content", element),
        span,
    )
}

/// Misplaced role attribute
pub fn a11y_misplaced_role(span: Span, element: &str) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::A11yMisplacedRole,
        format!("`<{}>` should not have a role attribute", element),
        span,
    )
}

/// Misplaced scope attribute
pub fn a11y_misplaced_scope(span: Span) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::A11yMisplacedScope,
        "The scope attribute should only be used on `<th>` elements",
        span,
    )
}

/// Abstract role should not be used
pub fn a11y_no_abstract_role(span: Span, role: &str) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::A11yNoAbstractRole,
        format!("Abstract role '{}' is forbidden", role),
        span,
    )
}

/// Redundant role
pub fn a11y_no_redundant_roles(span: Span, role: &str) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::A11yNoRedundantRoles,
        format!("Redundant role '{}' is already implicit", role),
        span,
    )
}

/// Positive tabindex should be avoided
pub fn a11y_positive_tabindex(span: Span) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::A11yPositiveTabindex,
        "Avoid positive tabindex values",
        span,
    )
}

/// Unknown ARIA attribute
pub fn a11y_unknown_aria_attribute(span: Span, attribute: &str, suggestion: Option<&str>) -> AnalysisWarning {
    let message = match suggestion {
        Some(s) => format!("Unknown aria attribute 'aria-{}'. Did you mean '{}'?", attribute, s),
        None => format!("Unknown aria attribute 'aria-{}'", attribute),
    };
    AnalysisWarning::new(WarningCode::A11yUnknownAriaAttribute, message, span)
}

/// Unknown role
pub fn a11y_unknown_role(span: Span, role: &str, suggestion: Option<&str>) -> AnalysisWarning {
    let message = match suggestion {
        Some(s) => format!("Unknown role '{}'. Did you mean '{}'?", role, s),
        None => format!("Unknown role '{}'", role),
    };
    AnalysisWarning::new(WarningCode::A11yUnknownRole, message, span)
}

/// Click events need key events
pub fn a11y_click_events_have_key_events(span: Span) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::A11yClickEventsHaveKeyEvents,
        "Visible, non-interactive elements with click handlers must have keyboard handlers",
        span,
    )
}

/// Mouse events need key events
pub fn a11y_mouse_events_have_key_events(span: Span, event: &str, keyboard_event: &str) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::A11yMouseEventsHaveKeyEvents,
        format!("`{}` event must be accompanied by `{}` event", event, keyboard_event),
        span,
    )
}

/// Media needs caption
pub fn a11y_media_has_caption(span: Span, element: &str) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::A11yMediaHasCaption,
        format!("`<{}>` elements must have a `<track kind=\"captions\">` element", element),
        span,
    )
}

/// Label missing associated control
pub fn a11y_label_has_associated_control(span: Span) -> AnalysisWarning {
    AnalysisWarning::new(
        WarningCode::A11yLabelHasAssociatedControl,
        "A form label must be associated with a control",
        span,
    )
}
