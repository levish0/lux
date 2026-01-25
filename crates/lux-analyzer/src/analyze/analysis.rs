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
}
