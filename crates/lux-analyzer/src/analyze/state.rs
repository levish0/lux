//! Analysis state passed to visitors.

use oxc_span::Span;
use rustc_hash::FxHashSet;

use super::analysis::ComponentAnalysis;
use crate::scope::ScopeId;

/// The type of AST being analyzed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AstType {
    /// Module script (<script context="module">)
    Module,
    /// Instance script (<script>)
    Instance,
    /// Template (HTML/Svelte markup)
    Template,
}

/// State passed to analysis visitors.
pub struct AnalysisState<'s, 'a> {
    /// Current scope
    pub scope: ScopeId,
    /// Reference to the analysis being built
    pub analysis: &'a mut ComponentAnalysis<'s>,
    /// Which part of the AST we're analyzing
    pub ast_type: AstType,
    /// Current fragment (if in template)
    pub fragment_span: Option<Span>,
    /// Parent element tag name (if any)
    pub parent_element: Option<&'a str>,
    /// Whether $props() has been seen
    pub has_props_rune: bool,
    /// Slots available in current component context
    pub component_slots: FxHashSet<String>,
    /// Current function nesting depth
    pub function_depth: u32,
    /// Current reactive statement (if in $: block)
    pub reactive_statement: Option<Span>,
    /// Depth at which $derived was seen (-1 if not in derived)
    pub derived_function_depth: i32,
    /// Whether we're currently inside a template expression (e.g., {expression})
    pub in_template_expression: bool,
    /// Current expression span being analyzed (for expression metadata tracking)
    /// When set, visitors update the expression metadata in analysis.expression_meta
    pub current_expression: Option<Span>,
}

impl<'s, 'a> AnalysisState<'s, 'a> {
    /// Creates a new analysis state.
    pub fn new(analysis: &'a mut ComponentAnalysis<'s>, scope: ScopeId, ast_type: AstType) -> Self {
        Self {
            scope,
            analysis,
            ast_type,
            fragment_span: None,
            parent_element: None,
            has_props_rune: false,
            component_slots: FxHashSet::default(),
            function_depth: 0,
            reactive_statement: None,
            derived_function_depth: -1,
            in_template_expression: false,
            current_expression: None,
        }
    }

    /// Returns true if we're analyzing the instance script.
    pub fn is_instance(&self) -> bool {
        self.ast_type == AstType::Instance
    }

    /// Returns true if we're analyzing the module script.
    pub fn is_module(&self) -> bool {
        self.ast_type == AstType::Module
    }

    /// Returns true if we're analyzing the template.
    pub fn is_template(&self) -> bool {
        self.ast_type == AstType::Template
    }

    /// Returns true if we're inside a function.
    pub fn in_function(&self) -> bool {
        self.function_depth > 0
    }

    /// Returns true if we're inside a $derived expression.
    pub fn in_derived(&self) -> bool {
        self.derived_function_depth >= 0
    }
}
