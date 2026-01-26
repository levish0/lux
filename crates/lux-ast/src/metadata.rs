/// Expression metadata — initialized during parsing with defaults,
/// populated during analysis phase.
/// Port of reference `phases/nodes.js` ExpressionMetadata class.
#[derive(Debug, Clone)]
pub struct ExpressionMetadata {
    pub has_state: bool,
    pub has_call: bool,
    pub has_await: bool,
    pub has_member_expression: bool,
    pub has_assignment: bool,
    // `dependencies` and `references` are Sets of Binding in the reference,
    // populated during analysis. We omit them here since they're analyzer concerns.
}

impl Default for ExpressionMetadata {
    fn default() -> Self {
        Self {
            has_state: false,
            has_call: false,
            has_await: false,
            has_member_expression: false,
            has_assignment: false,
        }
    }
}

/// Metadata for nodes that wrap an expression (ExpressionTag, HtmlTag, ConstTag, IfBlock, etc.)
#[derive(Debug, Clone, Default)]
pub struct ExpressionNodeMetadata {
    pub expression: ExpressionMetadata,
}

/// Metadata for RenderTag — has additional fields beyond ExpressionMetadata.
/// Port of reference: `{ expression: new ExpressionMetadata(), dynamic: false, arguments: [], path: [], snippets: new Set() }`
#[derive(Debug, Clone)]
pub struct RenderTagMetadata {
    pub expression: ExpressionMetadata,
    pub dynamic: bool,
    // `arguments`, `path`, `snippets` are analyzer concerns, omitted here.
}

impl Default for RenderTagMetadata {
    fn default() -> Self {
        Self {
            expression: ExpressionMetadata::default(),
            dynamic: false,
        }
    }
}

/// Metadata for SnippetBlock.
/// Port of reference: `{ can_hoist: false, sites: new Set() }`
#[derive(Debug, Clone)]
pub struct SnippetBlockMetadata {
    pub can_hoist: bool,
    // `sites` is analyzer concern, omitted here.
}

impl Default for SnippetBlockMetadata {
    fn default() -> Self {
        Self { can_hoist: false }
    }
}

// ============================================================================
// Block Metadata (populated during analysis)
// ============================================================================

/// Metadata for EachBlock, populated during analysis.
/// Reference: `phases/scope.js` EachBlock visitor
#[derive(Debug, Clone, Default)]
pub struct EachBlockMetadata {
    /// Expression metadata for the iterated expression
    pub expression: ExpressionMetadata,
    /// Whether this is a keyed each block
    pub keyed: bool,
    /// Whether this block contains a bind:group directive
    pub contains_group_binding: bool,
    /// Whether this block is controlled (e.g., by a parent)
    pub is_controlled: bool,
}

// ============================================================================
// Element Metadata (populated during analysis)
// ============================================================================

/// Metadata for RegularElement, populated during analysis.
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/RegularElement.js`
#[derive(Debug, Clone, Default)]
pub struct RegularElementMetadata {
    /// Whether this element has any spread attributes
    pub has_spread: bool,
    /// Whether this is an SVG element
    pub svg: bool,
    /// Whether this is a MathML element
    pub mathml: bool,
    /// Whether a synthetic value attribute node was created (for <option> elements)
    pub has_synthetic_value: bool,
}

/// Metadata for Component, populated during analysis.
#[derive(Debug, Clone, Default)]
pub struct ComponentMetadata {
    /// Whether this component has any spread attributes
    pub has_spread: bool,
    /// Whether this component is dynamic
    pub dynamic: bool,
}

// ============================================================================
// Fragment Metadata (populated during analysis)
// ============================================================================

/// Metadata for Fragment, populated during analysis.
#[derive(Debug, Clone, Default)]
pub struct FragmentMetadata {
    /// Whether this fragment is dynamic (requires reactive updates)
    pub dynamic: bool,
}
