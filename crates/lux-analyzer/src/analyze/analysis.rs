//! Analysis result structures.
//!
//! These structures contain the results of semantic analysis on Svelte components and modules.

use oxc_ast::ast::Program;
use oxc_span::Span;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::analyze::errors::AnalysisError;
use crate::analyze::warnings::AnalysisWarning;
use crate::scope::{BindingId, ScopeId, ScopeTree};

/// Analyzed JavaScript/TypeScript code (module or instance script).
#[derive(Debug)]
pub struct Js<'a> {
    /// The AST program
    pub ast: &'a Program<'a>,
    /// The root scope of this script
    pub scope: ScopeId,
    /// Map from AST node spans to their scopes
    pub scopes: FxHashMap<Span, ScopeId>,
    /// Whether this script contains top-level await
    pub has_await: bool,
}

/// Analyzed template fragment.
#[derive(Debug)]
pub struct Template<'a> {
    /// The AST fragment
    pub ast: &'a lux_ast::root::Fragment<'a>,
    /// The root scope of the template
    pub scope: ScopeId,
    /// Map from AST node spans to their scopes
    pub scopes: FxHashMap<Span, ScopeId>,
}

/// A reactive statement ($: label).
#[derive(Debug, Default)]
pub struct ReactiveStatement {
    /// Bindings assigned by this statement
    pub assignments: FxHashSet<BindingId>,
    /// Bindings this statement depends on
    pub dependencies: Vec<BindingId>,
}

// ============================================================================
// Block Metadata (populated during analysis)
// ============================================================================

/// Expression metadata for tracking reactivity.
/// Reference: `phases/nodes.js` ExpressionMetadata class
#[derive(Debug, Default)]
pub struct ExpressionMeta {
    /// True if the expression references state directly, or _might_ (via member/call expressions)
    pub has_state: bool,
    /// True if the expression involves a call expression
    pub has_call: bool,
    /// True if the expression contains `await`
    pub has_await: bool,
    /// True if the expression includes a member expression
    pub has_member_expression: bool,
    /// True if the expression includes an assignment or an update
    pub has_assignment: bool,
    /// Bindings that are referenced eagerly (not inside functions)
    pub dependencies: FxHashSet<BindingId>,
    /// All bindings that are referenced
    pub references: FxHashSet<BindingId>,
}

/// Metadata for EachBlock, populated during analysis.
/// Reference: `phases/scope.js` EachBlock visitor
#[derive(Debug, Default)]
pub struct EachBlockMeta {
    /// Expression metadata for the iterated expression
    pub expression: ExpressionMeta,
    /// Whether this is a keyed each block
    pub keyed: bool,
    /// Whether this block contains a bind:group directive
    pub contains_group_binding: bool,
    /// Whether this block is controlled (e.g., by a parent)
    pub is_controlled: bool,
    /// Transitive dependencies for legacy mode mutation tracking
    pub transitive_deps: FxHashSet<BindingId>,
}

/// Metadata for IfBlock, populated during analysis.
#[derive(Debug, Default)]
pub struct IfBlockMeta {
    /// Expression metadata for the test expression
    pub expression: ExpressionMeta,
}

/// Metadata for AwaitBlock, populated during analysis.
#[derive(Debug, Default)]
pub struct AwaitBlockMeta {
    /// Expression metadata for the awaited expression
    pub expression: ExpressionMeta,
}

/// Metadata for KeyBlock, populated during analysis.
#[derive(Debug, Default)]
pub struct KeyBlockMeta {
    /// Expression metadata for the key expression
    pub expression: ExpressionMeta,
}

/// Metadata for SnippetBlock, populated during analysis.
#[derive(Debug, Default)]
pub struct SnippetBlockMeta {
    /// Whether this snippet can be hoisted to module scope
    pub can_hoist: bool,
}

/// Metadata for RegularElement, populated during analysis.
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/RegularElement.js`
#[derive(Debug, Default)]
pub struct RegularElementMeta {
    /// Whether this element has any spread attributes
    pub has_spread: bool,
    /// Whether this is an SVG element
    pub svg: bool,
    /// Whether this is a MathML element
    pub mathml: bool,
    /// For <option> elements with single ExpressionTag child, stores the synthetic value node span
    pub synthetic_value_node: Option<Span>,
}

/// Metadata for Component, populated during analysis.
#[derive(Debug, Default)]
pub struct ComponentMeta {
    /// Whether this component has any spread attributes
    pub has_spread: bool,
}

/// CSS analysis result.
#[derive(Debug)]
pub struct CssAnalysis {
    /// The CSS AST (if present)
    pub ast: Option<Span>, // TODO: proper CSS AST reference
    /// The generated hash for scoping
    pub hash: String,
    /// Keyframe names defined in this stylesheet
    pub keyframes: Vec<String>,
    /// Whether the stylesheet contains :global rules
    pub has_global: bool,
}

impl Default for CssAnalysis {
    fn default() -> Self {
        Self {
            ast: None,
            hash: String::new(),
            keyframes: Vec::new(),
            has_global: false,
        }
    }
}

/// Analysis common to modules and components.
#[derive(Debug)]
pub struct Analysis {
    /// The component name
    pub name: String,
    /// Whether runes mode is enabled
    pub runes: bool,
    /// Whether immutable mode is enabled
    pub immutable: bool,
    /// Whether `$inspect.trace` is used
    pub tracing: bool,
    /// Whether accessors should be generated
    pub accessors: bool,
}

/// Full component analysis result.
#[derive(Debug)]
pub struct ComponentAnalysis<'s> {
    /// The source code being analyzed
    pub source: &'s str,
    /// Common analysis fields
    pub base: Analysis,
    /// The scope tree containing all bindings
    pub scope_tree: ScopeTree,
    /// Module script scope ID (if present)
    pub module_scope: Option<ScopeId>,
    /// Instance script scope ID (if present)
    pub instance_scope: Option<ScopeId>,
    /// Template scope ID
    pub template_scope: ScopeId,
    /// Whether runes mode is active (determined from usage)
    pub runes: bool,
    /// Whether this might be a runes component (for migration hints)
    pub maybe_runes: bool,
    /// Exported bindings (name -> alias)
    pub exports: Vec<Export>,
    /// Whether the component uses `$$props`
    pub uses_props: bool,
    /// Whether the component uses `$$restProps`
    pub uses_rest_props: bool,
    /// Whether the component uses `$$slots`
    pub uses_slots: bool,
    /// Whether the component uses component bindings (bind:this on components)
    pub uses_component_bindings: bool,
    /// Whether the component uses render tags ({@render ...})
    pub uses_render_tags: bool,
    /// Whether the component needs runtime context
    pub needs_context: bool,
    /// Whether mutation validation is needed
    pub needs_mutation_validation: bool,
    /// Whether props processing is needed
    pub needs_props: bool,
    /// First event directive found (for mixed syntax error)
    pub event_directive_span: Option<Span>,
    /// Whether event attributes are used
    pub uses_event_attributes: bool,
    /// Custom element configuration
    pub custom_element: bool,
    /// Whether styles should be injected via JS
    pub inject_styles: bool,
    /// Reactive statements ($:) and their dependencies
    pub reactive_statements: FxHashMap<Span, ReactiveStatement>,
    /// Slot names defined in this component
    pub slot_names: FxHashMap<String, Span>,
    /// CSS analysis
    pub css: CssAnalysis,
    /// Snippets declared in this component
    pub snippets: FxHashSet<Span>,
    /// Analysis errors collected during analysis
    pub errors: Vec<AnalysisError>,
    /// Analysis warnings collected during analysis
    pub warnings: Vec<AnalysisWarning>,

    // ========================================================================
    // Block Metadata (keyed by node span)
    // ========================================================================

    /// Metadata for EachBlock nodes
    pub each_block_meta: FxHashMap<Span, EachBlockMeta>,
    /// Metadata for IfBlock nodes
    pub if_block_meta: FxHashMap<Span, IfBlockMeta>,
    /// Metadata for AwaitBlock nodes
    pub await_block_meta: FxHashMap<Span, AwaitBlockMeta>,
    /// Metadata for KeyBlock nodes
    pub key_block_meta: FxHashMap<Span, KeyBlockMeta>,
    /// Metadata for SnippetBlock nodes
    pub snippet_block_meta: FxHashMap<Span, SnippetBlockMeta>,

    // ========================================================================
    // Element Metadata (keyed by node span)
    // ========================================================================

    /// Metadata for RegularElement nodes
    pub regular_element_meta: FxHashMap<Span, RegularElementMeta>,
    /// Metadata for Component nodes
    pub component_meta: FxHashMap<Span, ComponentMeta>,

    // ========================================================================
    // Fragment Metadata
    // ========================================================================

    /// Fragments that are marked as dynamic (by parent element/block span).
    /// Used by mark_subtree_dynamic to track which subtrees need dynamic handling.
    /// Reference: Fragment.metadata.dynamic in the JS implementation
    pub dynamic_fragments: FxHashSet<Span>,

    // ========================================================================
    // Expression Metadata
    // ========================================================================

    /// Metadata for template expressions, keyed by expression span.
    /// Reference: ExpressionTag, ConstTag, etc. have metadata.expression
    pub expression_meta: FxHashMap<Span, ExpressionMeta>,
}

/// An exported binding.
#[derive(Debug, Clone)]
pub struct Export {
    /// The local name
    pub name: String,
    /// The exported alias (if different from name)
    pub alias: Option<String>,
}

impl<'s> ComponentAnalysis<'s> {
    /// Creates a new ComponentAnalysis with default values.
    pub fn new(source: &'s str, name: String, scope_tree: ScopeTree, template_scope: ScopeId) -> Self {
        Self {
            source,
            base: Analysis {
                name: name.clone(),
                runes: false,
                immutable: false,
                tracing: false,
                accessors: false,
            },
            scope_tree,
            module_scope: None,
            instance_scope: None,
            template_scope,
            runes: false,
            maybe_runes: false,
            exports: Vec::new(),
            uses_props: false,
            uses_rest_props: false,
            uses_slots: false,
            uses_component_bindings: false,
            uses_render_tags: false,
            needs_context: false,
            needs_mutation_validation: false,
            needs_props: false,
            event_directive_span: None,
            uses_event_attributes: false,
            custom_element: false,
            inject_styles: false,
            reactive_statements: FxHashMap::default(),
            slot_names: FxHashMap::default(),
            css: CssAnalysis::default(),
            snippets: FxHashSet::default(),
            errors: Vec::new(),
            warnings: Vec::new(),
            each_block_meta: FxHashMap::default(),
            if_block_meta: FxHashMap::default(),
            await_block_meta: FxHashMap::default(),
            key_block_meta: FxHashMap::default(),
            snippet_block_meta: FxHashMap::default(),
            regular_element_meta: FxHashMap::default(),
            component_meta: FxHashMap::default(),
            dynamic_fragments: FxHashSet::default(),
            expression_meta: FxHashMap::default(),
        }
    }

    /// Adds an error to the analysis.
    pub fn error(&mut self, error: AnalysisError) {
        self.errors.push(error);
    }

    /// Adds a warning to the analysis.
    pub fn warning(&mut self, warning: AnalysisWarning) {
        self.warnings.push(warning);
    }

    /// Returns true if there are any errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Returns true if the fragment at the given span is marked as dynamic.
    /// Used by transform phase to determine if a fragment needs dynamic handling.
    pub fn is_fragment_dynamic(&self, span: Span) -> bool {
        self.dynamic_fragments.contains(&span)
    }

    /// Marks a fragment as dynamic.
    pub fn mark_fragment_dynamic(&mut self, span: Span) {
        self.dynamic_fragments.insert(span);
    }

    /// Gets or creates expression metadata for a given span.
    pub fn get_or_create_expression_meta(&mut self, span: Span) -> &mut ExpressionMeta {
        self.expression_meta.entry(span).or_default()
    }

    /// Gets expression metadata for a given span (if it exists).
    pub fn get_expression_meta(&self, span: Span) -> Option<&ExpressionMeta> {
        self.expression_meta.get(&span)
    }

    /// Updates expression metadata: sets has_state to true.
    pub fn mark_expression_has_state(&mut self, span: Span) {
        self.get_or_create_expression_meta(span).has_state = true;
    }

    /// Updates expression metadata: sets has_call to true.
    pub fn mark_expression_has_call(&mut self, span: Span) {
        self.get_or_create_expression_meta(span).has_call = true;
    }

    /// Updates expression metadata: sets has_await to true.
    pub fn mark_expression_has_await(&mut self, span: Span) {
        self.get_or_create_expression_meta(span).has_await = true;
    }

    /// Updates expression metadata: sets has_member_expression to true.
    pub fn mark_expression_has_member(&mut self, span: Span) {
        self.get_or_create_expression_meta(span).has_member_expression = true;
    }

    /// Updates expression metadata: sets has_assignment to true.
    pub fn mark_expression_has_assignment(&mut self, span: Span) {
        self.get_or_create_expression_meta(span).has_assignment = true;
    }

    /// Adds a dependency to expression metadata.
    pub fn add_expression_dependency(&mut self, span: Span, binding_id: BindingId) {
        let meta = self.get_or_create_expression_meta(span);
        meta.dependencies.insert(binding_id);
        meta.references.insert(binding_id);
    }
}
