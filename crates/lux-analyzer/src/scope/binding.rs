//! Binding types for Svelte semantic analysis.

use super::ScopeId;
use oxc_span::Span;

// ============================================================================
// Binding & Declaration Kinds
// ============================================================================

/// The kind of binding, determining how it behaves in the reactivity system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingKind {
    /// A variable that is not in any way special
    Normal,
    /// A normal prop (possibly reassigned or mutated)
    Prop,
    /// A prop one can `bind:` to (possibly reassigned or mutated)
    BindableProp,
    /// A rest prop
    RestProp,
    /// A `$state.raw()` variable
    RawState,
    /// A `$state()` deeply reactive variable
    State,
    /// A `$derived()` variable
    Derived,
    /// An each block parameter
    Each,
    /// A snippet parameter
    Snippet,
    /// A `$store` subscription
    StoreSub,
    /// A `$:` declaration (legacy)
    LegacyReactive,
    /// A binding declared in the template, e.g. in an `await` block or `const` tag
    Template,
    /// A binding whose value is known to be static (i.e. each index)
    Static,
}

/// How the binding was declared.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclarationKind {
    Var,
    Let,
    Const,
    Using,
    AwaitUsing,
    Function,
    Import,
    Param,
    RestParam,
    /// Synthetic bindings created by the compiler (e.g. $$props, $$restProps)
    Synthetic,
}

// ============================================================================
// Binding Initial & References
// ============================================================================

/// What the binding was initialized with.
#[derive(Debug, Clone, Copy)]
pub enum BindingInitial {
    None,
    Expression(Span),
    FunctionDeclaration(Span),
    ClassDeclaration(Span),
    ImportDeclaration(Span),
    EachBlock(Span),
    SnippetBlock(Span),
}

/// A reference to an identifier.
#[derive(Debug, Clone)]
pub struct Reference {
    /// The span of the identifier
    pub span: Span,
    // TODO: path tracking if needed
}

/// An assignment to a binding.
#[derive(Debug, Clone)]
pub struct Assignment {
    /// The span of the assignment
    pub value_span: Span,
    /// The scope where the assignment occurred
    pub scope: ScopeId,
}

// ============================================================================
// Binding
// ============================================================================

/// Represents a variable binding in a scope.
#[derive(Debug)]
pub struct Binding {
    /// The scope this binding belongs to
    pub scope: ScopeId,
    /// The identifier node that declares this binding
    pub node_span: Span,
    /// The name of the binding
    pub name: String,
    /// What kind of binding this is
    pub kind: BindingKind,
    /// How it was declared
    pub declaration_kind: DeclarationKind,
    /// What it was initialized with
    pub initial: BindingInitial,
    /// All references to this binding
    pub references: Vec<Reference>,
    /// All assignments to this binding
    pub assignments: Vec<Assignment>,
    /// For legacy_reactive: its reactive dependencies
    pub legacy_dependencies: Vec<super::BindingId>,
    /// Legacy props: the alias name if any (e.g., `class` in `export { klass as class }`)
    pub prop_alias: Option<String>,
    /// Whether this binding has been mutated (e.g., `obj.prop = value`)
    pub mutated: bool,
    /// Whether this binding has been reassigned (e.g., `x = value`)
    pub reassigned: bool,
}

impl Binding {
    pub fn new(
        scope: ScopeId,
        node_span: Span,
        name: String,
        kind: BindingKind,
        declaration_kind: DeclarationKind,
    ) -> Self {
        Self {
            scope,
            node_span,
            name,
            kind,
            declaration_kind,
            initial: BindingInitial::None,
            references: Vec::new(),
            assignments: Vec::new(),
            legacy_dependencies: Vec::new(),
            prop_alias: None,
            mutated: false,
            reassigned: false,
        }
    }

    /// Returns true if this binding has been updated (mutated or reassigned).
    #[inline]
    pub fn updated(&self) -> bool {
        self.mutated || self.reassigned
    }

    /// Returns true if this binding is a function that hasn't been reassigned.
    pub fn is_function(&self) -> bool {
        if self.updated() {
            return false;
        }
        matches!(
            self.initial,
            BindingInitial::FunctionDeclaration(_)
                | BindingInitial::Expression(_) // TODO: check if it's actually a function expression
        )
    }
}
