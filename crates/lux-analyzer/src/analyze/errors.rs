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

    // Const tag errors
    ConstTagInvalidPlacement,

    // Slot element errors
    SlotElementDeprecated,
    SlotElementInvalidName,
    SlotElementInvalidNameDefault,
    SlotElementInvalidAttribute,

    // Component errors
    ComponentInvalidDirective,
    EventHandlerInvalidComponentModifier,

    // Svelte special element errors
    SvelteSelfInvalidPlacement,
    SvelteBodyIllegalAttribute,
    SvelteWindowIllegalAttribute,
    SvelteDocumentIllegalAttribute,
    SvelteHeadIllegalAttribute,
    SvelteFragmentInvalidPlacement,
    SvelteFragmentInvalidAttribute,
    SvelteBoundaryInvalidAttribute,
    SvelteBoundaryInvalidAttributeValue,

    // Title element errors
    TitleIllegalAttribute,
    TitleInvalidContent,

    // Directive errors
    StyleDirectiveInvalidModifier,
    LetDirectiveInvalidPlacement,

    // Reactive statement errors
    LegacyReactiveStatementInvalid,

    // Export errors
    ModuleIllegalDefaultExport,
    LegacyExportInvalid,
    DerivedInvalidExport,
    StateInvalidExport,

    // Props errors
    PropsInvalidIdentifier,
    PropsInvalidPattern,
    PropsIllegalName,

    // Rune errors in non-runes mode
    RuneInvalidUsage,

    // Assignment errors
    ConstantAssignment,
    ConstantBinding,
    EachItemInvalidAssignment,
    SnippetParameterAssignment,
    DollarBindingInvalid,
    DollarPrefixInvalid,

    // Import errors
    ImportSvelteInternalForbidden,
    RunesModeInvalidImport,

    // Await errors
    ExperimentalAsync,
    LegacyAwaitInvalid,

    // Legacy component errors
    LegacyComponentCreation,

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

// =============================================================================
// Const tag errors
// =============================================================================

pub fn const_tag_invalid_placement(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::ConstTagInvalidPlacement,
        "`{@const ...}` must be the immediate child of `{#if ...}`, `{:else if ...}`, `{:else}`, `{#each ...}`, `{:then ...}`, `{:catch ...}`, `{#snippet ...}` or a `<Component />`",
        span,
    )
}

// =============================================================================
// Slot element errors
// =============================================================================

pub fn slot_element_invalid_name(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::SlotElementInvalidName,
        "`name` attribute must be a static value",
        span,
    )
}

pub fn slot_element_invalid_name_default(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::SlotElementInvalidNameDefault,
        "The default slot cannot have a name",
        span,
    )
}

pub fn slot_element_invalid_attribute(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::SlotElementInvalidAttribute,
        "`<slot>` can only have a `name` attribute and `let:` directives",
        span,
    )
}

// =============================================================================
// Component errors
// =============================================================================

pub fn component_invalid_directive(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::ComponentInvalidDirective,
        "This directive is not allowed on components",
        span,
    )
}

pub fn event_handler_invalid_component_modifier(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::EventHandlerInvalidComponentModifier,
        "Event handlers on components can only have the `once` modifier",
        span,
    )
}

// =============================================================================
// Svelte special element errors
// =============================================================================

pub fn svelte_self_invalid_placement(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::SvelteSelfInvalidPlacement,
        "`<svelte:self>` can only appear inside an if block, each block, component, or snippet",
        span,
    )
}

pub fn svelte_body_illegal_attribute(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::SvelteBodyIllegalAttribute,
        "`<svelte:body>` can only have event handlers",
        span,
    )
}

pub fn svelte_window_illegal_attribute(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::SvelteWindowIllegalAttribute,
        "`<svelte:window>` can only have event handlers and bindings",
        span,
    )
}

pub fn svelte_document_illegal_attribute(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::SvelteDocumentIllegalAttribute,
        "`<svelte:document>` can only have event handlers and bindings",
        span,
    )
}

pub fn svelte_head_illegal_attribute(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::SvelteHeadIllegalAttribute,
        "`<svelte:head>` cannot have attributes",
        span,
    )
}

pub fn svelte_fragment_invalid_placement(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::SvelteFragmentInvalidPlacement,
        "`<svelte:fragment>` can only appear as a direct child of a component",
        span,
    )
}

pub fn svelte_fragment_invalid_attribute(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::SvelteFragmentInvalidAttribute,
        "`<svelte:fragment>` can only have `slot` attribute and `let:` directives",
        span,
    )
}

pub fn svelte_boundary_invalid_attribute(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::SvelteBoundaryInvalidAttribute,
        "`<svelte:boundary>` can only have `onerror`, `failed`, and `pending` attributes",
        span,
    )
}

pub fn svelte_boundary_invalid_attribute_value(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::SvelteBoundaryInvalidAttributeValue,
        "`<svelte:boundary>` attributes must have an expression value",
        span,
    )
}

// =============================================================================
// Title element errors
// =============================================================================

pub fn title_illegal_attribute(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::TitleIllegalAttribute,
        "`<title>` cannot have attributes",
        span,
    )
}

pub fn title_invalid_content(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::TitleInvalidContent,
        "`<title>` can only contain text and expression tags",
        span,
    )
}

// =============================================================================
// Directive errors
// =============================================================================

pub fn style_directive_invalid_modifier(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::StyleDirectiveInvalidModifier,
        "`style:` directive can only use the `important` modifier",
        span,
    )
}

pub fn let_directive_invalid_placement(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::LetDirectiveInvalidPlacement,
        "`let:` directive can only be used on components, `<slot>`, `<svelte:element>`, `<svelte:component>`, `<svelte:self>`, or `<svelte:fragment>`",
        span,
    )
}

// =============================================================================
// Reactive statement errors
// =============================================================================

pub fn legacy_reactive_statement_invalid(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::LegacyReactiveStatementInvalid,
        "`$:` is not allowed in runes mode. Use `$derived` or `$effect` instead",
        span,
    )
}

// =============================================================================
// Export errors
// =============================================================================

pub fn module_illegal_default_export(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::ModuleIllegalDefaultExport,
        "A component cannot have a default export",
        span,
    )
}

pub fn legacy_export_invalid(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::LegacyExportInvalid,
        "`export let` is not allowed in runes mode. Use `$props()` instead",
        span,
    )
}

pub fn derived_invalid_export(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::DerivedInvalidExport,
        "Cannot export derived state",
        span,
    )
}

pub fn state_invalid_export(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::StateInvalidExport,
        "Cannot export state if it is reassigned",
        span,
    )
}

// =============================================================================
// Props errors
// =============================================================================

pub fn props_invalid_identifier(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::PropsInvalidIdentifier,
        "`$props()` must be assigned to an object destructuring pattern or an identifier",
        span,
    )
}

pub fn props_invalid_pattern(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::PropsInvalidPattern,
        "`$props()` patterns can only have computed keys with literal values",
        span,
    )
}

pub fn props_illegal_name(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::PropsIllegalName,
        "Property names starting with `$$` are reserved",
        span,
    )
}

// =============================================================================
// Rune usage errors
// =============================================================================

pub fn rune_invalid_usage(span: Span, name: &str) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::RuneInvalidUsage,
        format!(
            "`{}` is only available inside `.svelte` files with runes mode enabled",
            name
        ),
        span,
    )
}

// =============================================================================
// Assignment errors
// =============================================================================

pub fn constant_assignment(span: Span, thing: &str) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::ConstantAssignment,
        format!("Cannot assign to {}", thing),
        span,
    )
}

pub fn constant_binding(span: Span, thing: &str) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::ConstantBinding,
        format!("Cannot bind to {}", thing),
        span,
    )
}

pub fn each_item_invalid_assignment(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::EachItemInvalidAssignment,
        "Cannot reassign or bind to each block argument in runes mode. Use the array and index variables instead (e.g. `array[i] = value` instead of `entry = value`)",
        span,
    )
}

pub fn snippet_parameter_assignment(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::SnippetParameterAssignment,
        "Cannot reassign or bind to snippet parameter",
        span,
    )
}

pub fn dollar_binding_invalid(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::DollarBindingInvalid,
        "The $ name is reserved, and cannot be used for variables and imports",
        span,
    )
}

pub fn dollar_prefix_invalid(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::DollarPrefixInvalid,
        "The $ prefix is reserved, and cannot be used for variables and imports",
        span,
    )
}

pub fn props_id_invalid_placement(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::PropsIdInvalidPlacement,
        "`$props.id()` can only be used at the top level of the instance script as a variable declaration initializer",
        span,
    )
}

// =============================================================================
// Import errors
// =============================================================================

pub fn import_svelte_internal_forbidden(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::ImportSvelteInternalForbidden,
        "Importing from `svelte/internal/*` is forbidden. It contains private APIs that your app should not use",
        span,
    )
}

pub fn runes_mode_invalid_import(span: Span, name: &str) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::RunesModeInvalidImport,
        format!("`{}` is not available in runes mode. Use `$effect.pre` instead", name),
        span,
    )
}

// =============================================================================
// Await errors
// =============================================================================

pub fn experimental_async(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::ExperimentalAsync,
        "Top-level `await` and `await` in template expressions require `experimental.async` to be enabled",
        span,
    )
}

pub fn legacy_await_invalid(span: Span) -> AnalysisError {
    AnalysisError::new(
        ErrorCode::LegacyAwaitInvalid,
        "`await` in template expressions is only allowed in runes mode",
        span,
    )
}
