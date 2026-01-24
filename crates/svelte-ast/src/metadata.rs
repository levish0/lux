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
