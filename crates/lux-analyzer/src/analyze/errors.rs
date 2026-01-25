//! Analysis errors.
//!
//! These are compile-time errors detected during semantic analysis.

use oxc_span::Span;

/// An analysis error.
#[derive(Debug, Clone)]
pub struct AnalysisError {
    pub code: ErrorCode,
    pub message: String,
    pub span: Span,
}

impl AnalysisError {
    pub fn new(code: ErrorCode, message: impl Into<String>, span: Span) -> Self {
        Self {
            code,
            message: message.into(),
            span,
        }
    }
}

/// Error codes for analysis errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    // Rune errors
    RuneInvalidArguments,
    RuneInvalidArgumentsLength,
    RuneInvalidSpread,
    RuneInvalidName,
    RuneInvalidComputedProperty,
    RuneMissingParentheses,
    RuneRenamed,
    RuneRemoved,

    // Placement errors
    PropsInvalidPlacement,
    PropsDuplicate,
    PropsIdInvalidPlacement,
    StateInvalidPlacement,
    EffectInvalidPlacement,
    BindableInvalidLocation,
    HostInvalidPlacement,
    InspectTraceInvalidPlacement,
    InspectTraceGenerator,

    // Binding errors
    BindInvalidTarget,
    BindInvalidName,
    BindInvalidExpression,
    BindInvalidValue,
    BindGroupInvalidExpression,
    BindGroupInvalidSnippetParameter,
    BindInvalidParens,

    // Attribute errors
    AttributeInvalidType,
    AttributeInvalidMultiple,
    AttributeContenteditableMissing,
    AttributeContenteditableDynamic,

    // Attribute errors
    AttributeInvalidName,
    AttributeInvalidEventHandler,
    AttributeInvalidSequenceExpression,
    AttributeUnquotedSequence,
    SlotAttributeInvalid,
    SlotAttributeInvalidPlacement,
    SlotAttributeDuplicate,
    SlotDefaultDuplicate,

    // Element errors
    TextareaInvalidContent,
    NodeInvalidPlacement,
    SvelteMetaInvalidContent,

    // Transition/Animation errors
    AnimationInvalidPlacement,
    AnimationMissingKey,
    AnimationDuplicate,
    TransitionDuplicate,
    TransitionConflict,

    // Event handler errors
    EventHandlerInvalidModifier,
    EventHandlerInvalidModifierCombination,

    // Block errors
    EachKeyWithoutAs,
    BlockUnexpectedCharacter,

    // Snippet errors
    SnippetInvalidRestParameter,
    SnippetShadowingProp,
    SnippetConflict,

    // Render tag errors
    RenderTagInvalidSpreadArgument,
    RenderTagInvalidCallExpression,

    // Other errors
    InvalidArgumentsUsage,
    IllegalAwaitExpression,
}

// Error constructors

pub fn rune_invalid_spread(span: Span, rune: &str) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::RuneInvalidSpread,
        format!("Cannot use spread arguments in `{}`", rune),
        span,
    )
}

pub fn rune_invalid_arguments(span: Span, rune: &str) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::RuneInvalidArguments,
        format!("`{}` does not take any arguments", rune),
        span,
    )
}

pub fn rune_invalid_arguments_length(span: Span, rune: &str, expected: &str) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::RuneInvalidArgumentsLength,
        format!("`{}` must be called with {}", rune, expected),
        span,
    )
}

pub fn rune_invalid_name(span: Span, name: &str) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::RuneInvalidName,
        format!("`{}` is not a valid rune", name),
        span,
    )
}

pub fn rune_invalid_computed_property(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::RuneInvalidComputedProperty,
        "Cannot use computed property access with runes",
        span,
    )
}

pub fn rune_missing_parentheses(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::RuneMissingParentheses,
        "Rune must be called with parentheses",
        span,
    )
}

pub fn rune_renamed(span: Span, old: &str, new: &str) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::RuneRenamed,
        format!("`{}` has been renamed to `{}`", old, new),
        span,
    )
}

pub fn rune_removed(span: Span, name: &str) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::RuneRemoved,
        format!("`{}` has been removed", name),
        span,
    )
}

pub fn props_invalid_placement(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::PropsInvalidPlacement,
        "`$props()` can only be used at the top level of the instance script as a variable declaration initializer",
        span,
    )
}

pub fn props_duplicate(span: Span, rune: &str) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::PropsDuplicate,
        format!("`{}` can only be used once per component", rune),
        span,
    )
}

pub fn state_invalid_placement(span: Span, rune: &str) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::StateInvalidPlacement,
        format!("`{}` can only be used as a variable declaration initializer or a class property", rune),
        span,
    )
}

pub fn effect_invalid_placement(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::EffectInvalidPlacement,
        "`$effect()` can only be used as an expression statement",
        span,
    )
}

pub fn bindable_invalid_location(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::BindableInvalidLocation,
        "`$bindable()` can only be used inside a `$props()` declaration",
        span,
    )
}

pub fn host_invalid_placement(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::HostInvalidPlacement,
        "`$host()` can only be used inside a custom element component",
        span,
    )
}

pub fn inspect_trace_invalid_placement(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::InspectTraceInvalidPlacement,
        "`$inspect.trace()` must be the first statement inside a function body",
        span,
    )
}

pub fn inspect_trace_generator(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::InspectTraceGenerator,
        "`$inspect.trace()` cannot be used inside a generator function",
        span,
    )
}

pub fn bind_invalid_target(span: Span, name: &str, valid: &str) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::BindInvalidTarget,
        format!("`bind:{}` can only be used with {}", name, valid),
        span,
    )
}

pub fn bind_invalid_name(span: Span, name: &str, hint: Option<&str>) -> AnalysisError {
    let msg = match hint {
        Some(h) => format!("`bind:{}` is not a valid binding. {}", name, h),
        None => format!("`bind:{}` is not a valid binding", name),
    };
    AnalysisError::new(ErrorCode::BindInvalidName, msg, span)
}

pub fn bind_invalid_expression(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::BindInvalidExpression,
        "bind: value must be an identifier or a member expression",
        span,
    )
}

pub fn bind_invalid_value(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::BindInvalidValue,
        "Can only bind to state or props",
        span,
    )
}

pub fn invalid_arguments_usage(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::InvalidArgumentsUsage,
        "`arguments` is not allowed outside of functions",
        span,
    )
}

pub fn illegal_await_expression(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::IllegalAwaitExpression,
        "`await` is not allowed in this context",
        span,
    )
}

// =============================================================================
// Attribute errors
// =============================================================================

pub fn attribute_invalid_name(span: Span, name: &str) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::AttributeInvalidName,
        format!("`{}` is not a valid attribute name", name),
        span,
    )
}

pub fn attribute_invalid_event_handler(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::AttributeInvalidEventHandler,
        "Event attribute must have a value",
        span,
    )
}

pub fn attribute_invalid_sequence_expression(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::AttributeInvalidSequenceExpression,
        "Sequence expressions are not allowed in unparenthesized attribute values",
        span,
    )
}

pub fn attribute_unquoted_sequence(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::AttributeUnquotedSequence,
        "Attribute values containing multiple expressions must be quoted",
        span,
    )
}

pub fn slot_attribute_invalid(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::SlotAttributeInvalid,
        "`slot` attribute must be a static value",
        span,
    )
}

pub fn slot_attribute_invalid_placement(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::SlotAttributeInvalidPlacement,
        "`slot` can only be used on elements that are direct children of a component",
        span,
    )
}

pub fn slot_attribute_duplicate(span: Span, name: &str, component: &str) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::SlotAttributeDuplicate,
        format!(
            "Duplicate slot `{}` on `<{}>`",
            name, component
        ),
        span,
    )
}

pub fn slot_default_duplicate(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::SlotDefaultDuplicate,
        "Cannot have both default slot content and explicit default slot",
        span,
    )
}

// =============================================================================
// Element errors
// =============================================================================

pub fn textarea_invalid_content(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::TextareaInvalidContent,
        "`<textarea>` cannot have both a `value` attribute and content",
        span,
    )
}

pub fn node_invalid_placement(span: Span, message: &str) -> AnalysisError {
    AnalysisError::new(ErrorCode::NodeInvalidPlacement, message.to_string(), span)
}

pub fn svelte_meta_invalid_content(span: Span, name: &str) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::SvelteMetaInvalidContent,
        format!("`<{}>` cannot have children", name),
        span,
    )
}

// =============================================================================
// Transition/Animation errors
// =============================================================================

pub fn animation_invalid_placement(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::AnimationInvalidPlacement,
        "An element with `animate` must be the immediate child of a keyed each block",
        span,
    )
}

pub fn animation_missing_key(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::AnimationMissingKey,
        "An element with `animate` must be inside a keyed each block. Did you forget to add a key?",
        span,
    )
}

pub fn animation_duplicate(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::AnimationDuplicate,
        "An element can only have one `animate` directive",
        span,
    )
}

pub fn transition_duplicate(span: Span, kind: &str) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::TransitionDuplicate,
        format!("An element can only have one `{}` directive", kind),
        span,
    )
}

pub fn transition_conflict(span: Span, a: &str, b: &str) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::TransitionConflict,
        format!(
            "An element cannot have both `{}` and `{}` directives",
            a, b
        ),
        span,
    )
}

// =============================================================================
// Event handler errors
// =============================================================================

pub fn event_handler_invalid_modifier(span: Span, valid_list: &str) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::EventHandlerInvalidModifier,
        format!("Event modifier must be one of: {}", valid_list),
        span,
    )
}

pub fn event_handler_invalid_modifier_combination(
    span: Span,
    a: &str,
    b: &str,
) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::EventHandlerInvalidModifierCombination,
        format!(
            "The `{}` and `{}` modifiers cannot be used together",
            a, b
        ),
        span,
    )
}

// =============================================================================
// Block errors
// =============================================================================

pub fn each_key_without_as(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::EachKeyWithoutAs,
        "Keyed each block requires an `as` clause",
        span,
    )
}

pub fn block_unexpected_character(span: Span, expected: &str) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::BlockUnexpectedCharacter,
        format!("Expected `{}`", expected),
        span,
    )
}

// =============================================================================
// Snippet errors
// =============================================================================

pub fn snippet_invalid_rest_parameter(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::SnippetInvalidRestParameter,
        "Snippets do not support rest parameters. Use an array instead",
        span,
    )
}

pub fn snippet_shadowing_prop(span: Span, name: &str) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::SnippetShadowingProp,
        format!(
            "The snippet `{}` is shadowing a prop with the same name",
            name
        ),
        span,
    )
}

pub fn snippet_conflict(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::SnippetConflict,
        "Cannot use `{#snippet children(...)}` if the component has content outside snippets",
        span,
    )
}

// =============================================================================
// Render tag errors
// =============================================================================

pub fn render_tag_invalid_spread_argument(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::RenderTagInvalidSpreadArgument,
        "Render tag arguments cannot use spread syntax",
        span,
    )
}

pub fn render_tag_invalid_call_expression(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::RenderTagInvalidCallExpression,
        "Render tag callee cannot be `.bind()`, `.apply()`, or `.call()`",
        span,
    )
}
