use rustc_hash::FxHashSet;

use crate::common::BindingId;

#[derive(Debug, Clone, Default)]
pub struct ExpressionMetadata {
    pub has_state: bool,
    pub has_call: bool,
    pub has_await: bool,
    pub has_member_expression: bool,
    pub has_assignment: bool,
    pub dependencies: FxHashSet<BindingId>,
    pub references: FxHashSet<BindingId>,
}

#[derive(Debug, Clone, Default)]
pub struct RegularElementMetadata {
    pub svg: bool,
    pub mathml: bool,
    pub has_spread: bool,
    pub scoped: bool,
}

#[derive(Debug, Clone)]
pub struct ComponentMetadata {
    pub expression: ExpressionMetadata,
    pub dynamic: bool,
}

#[derive(Debug, Clone)]
pub struct EachBlockMetadata {
    pub expression: ExpressionMetadata,
    pub keyed: bool,
    pub contains_group_binding: bool,
    pub is_controlled: bool,
}

#[derive(Debug, Clone, Default)]
pub struct SnippetBlockMetadata {
    pub can_hoist: bool,
}

#[derive(Debug, Clone)]
pub struct RenderTagMetadata {
    pub expression: ExpressionMetadata,
    pub dynamic: bool,
}

#[derive(Debug, Clone, Default)]
pub struct AttributeMetadata {
    pub delegated: bool,
    pub needs_clsx: bool,
}

#[derive(Debug, Clone)]
pub struct BindDirectiveMetadata {
    pub expression: ExpressionMetadata,
    pub binding_group_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SvelteElementMetadata {
    pub expression: ExpressionMetadata,
    pub scoped: bool,
}
