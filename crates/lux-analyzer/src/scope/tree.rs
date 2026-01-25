//! Scope tree for Svelte semantic analysis.

use super::binding::{Binding, BindingKind, DeclarationKind, Reference};
use lux_utils::is_reserved;
use oxc_span::Span;
use rustc_hash::{FxHashMap, FxHashSet};

// ============================================================================
// IDs
// ============================================================================

/// A unique identifier for a scope in the scope tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(u32);

impl ScopeId {
    pub const ROOT: ScopeId = ScopeId(0);

    #[inline]
    pub fn new(index: u32) -> Self {
        Self(index)
    }

    #[inline]
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// A unique identifier for a binding in the scope tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BindingId(u32);

impl BindingId {
    #[inline]
    pub fn new(index: u32) -> Self {
        Self(index)
    }

    #[inline]
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

// ============================================================================
// Scope
// ============================================================================

/// A lexical scope in the program.
#[derive(Debug)]
pub struct Scope {
    /// Parent scope, if any
    pub parent: Option<ScopeId>,
    /// Whether `var` declarations are contained by this scope (porous = true means they leak)
    porous: bool,
    /// The depth of this scope in terms of function nesting
    pub function_depth: u32,
    /// Bindings declared in this scope (name -> binding id)
    pub declarations: FxHashMap<String, BindingId>,
    /// References made in this scope (name -> list of references)
    pub references: FxHashMap<String, Vec<Reference>>,
}

impl Scope {
    pub fn new(parent: Option<ScopeId>, porous: bool, function_depth: u32) -> Self {
        Self {
            parent,
            porous,
            function_depth,
            declarations: FxHashMap::default(),
            references: FxHashMap::default(),
        }
    }

    /// Returns true if this scope is porous (var declarations leak to parent).
    #[inline]
    pub fn is_porous(&self) -> bool {
        self.porous
    }
}

// ============================================================================
// ScopeTree
// ============================================================================

/// The scope tree containing all scopes and bindings for a compilation unit.
#[derive(Debug)]
pub struct ScopeTree {
    /// All scopes, indexed by ScopeId
    scopes: Vec<Scope>,
    /// All bindings, indexed by BindingId
    bindings: Vec<Binding>,
    /// Map from AST node span to scope id
    node_scopes: FxHashMap<Span, ScopeId>,
    /// Set of all names that conflict (used for generating unique names)
    pub conflicts: FxHashSet<String>,
}

impl ScopeTree {
    pub fn new() -> Self {
        let mut tree = Self {
            scopes: Vec::new(),
            bindings: Vec::new(),
            node_scopes: FxHashMap::default(),
            conflicts: FxHashSet::default(),
        };
        // Create root scope
        tree.scopes.push(Scope::new(None, false, 0));
        tree
    }

    /// Returns the root scope id.
    #[inline]
    pub fn root_scope_id(&self) -> ScopeId {
        ScopeId::ROOT
    }

    /// Returns a reference to a scope by id.
    #[inline]
    pub fn get_scope(&self, id: ScopeId) -> &Scope {
        &self.scopes[id.index()]
    }

    /// Returns a mutable reference to a scope by id.
    #[inline]
    pub fn get_scope_mut(&mut self, id: ScopeId) -> &mut Scope {
        &mut self.scopes[id.index()]
    }

    /// Returns a reference to a binding by id.
    #[inline]
    pub fn get_binding(&self, id: BindingId) -> &Binding {
        &self.bindings[id.index()]
    }

    /// Returns a mutable reference to a binding by id.
    #[inline]
    pub fn get_binding_mut(&mut self, id: BindingId) -> &mut Binding {
        &mut self.bindings[id.index()]
    }

    /// Creates a new child scope.
    pub fn create_child_scope(&mut self, parent: ScopeId, porous: bool) -> ScopeId {
        let parent_scope = &self.scopes[parent.index()];
        let function_depth = if porous {
            parent_scope.function_depth
        } else {
            parent_scope.function_depth + 1
        };

        let id = ScopeId::new(self.scopes.len() as u32);
        self.scopes
            .push(Scope::new(Some(parent), porous, function_depth));
        id
    }

    /// Associates an AST node (by span) with a scope.
    pub fn set_node_scope(&mut self, span: Span, scope_id: ScopeId) {
        self.node_scopes.insert(span, scope_id);
    }

    /// Gets the scope associated with an AST node.
    pub fn get_node_scope(&self, span: Span) -> Option<ScopeId> {
        self.node_scopes.get(&span).copied()
    }

    /// Declares a binding in the given scope.
    /// Returns the binding id.
    pub fn declare(
        &mut self,
        scope_id: ScopeId,
        name: String,
        node_span: Span,
        kind: BindingKind,
        declaration_kind: DeclarationKind,
    ) -> BindingId {
        // Handle var hoisting: if scope is porous and declaration is var,
        // declare in parent scope instead
        let target_scope_id = if declaration_kind == DeclarationKind::Var {
            self.find_var_scope(scope_id)
        } else if declaration_kind == DeclarationKind::Import {
            // Imports also hoist to the nearest non-porous scope
            self.find_var_scope(scope_id)
        } else {
            scope_id
        };

        let binding_id = BindingId::new(self.bindings.len() as u32);
        let binding = Binding::new(target_scope_id, node_span, name.clone(), kind, declaration_kind);
        self.bindings.push(binding);

        let scope = &mut self.scopes[target_scope_id.index()];
        scope.declarations.insert(name.clone(), binding_id);
        self.conflicts.insert(name);

        binding_id
    }

    /// Finds the nearest non-porous scope for var declarations.
    fn find_var_scope(&self, scope_id: ScopeId) -> ScopeId {
        let mut current = scope_id;
        loop {
            let scope = &self.scopes[current.index()];
            if !scope.porous {
                return current;
            }
            match scope.parent {
                Some(parent) => current = parent,
                None => return current,
            }
        }
    }

    /// Looks up a binding by name, starting from the given scope and walking up.
    pub fn get(&self, scope_id: ScopeId, name: &str) -> Option<BindingId> {
        let mut current = Some(scope_id);
        while let Some(id) = current {
            let scope = &self.scopes[id.index()];
            if let Some(&binding_id) = scope.declarations.get(name) {
                return Some(binding_id);
            }
            current = scope.parent;
        }
        None
    }

    /// Records a reference to an identifier.
    pub fn reference(&mut self, scope_id: ScopeId, name: String, span: Span) {
        let reference = Reference { span };

        // Add to scope's references
        let scope = &mut self.scopes[scope_id.index()];
        scope
            .references
            .entry(name.clone())
            .or_default()
            .push(reference.clone());

        // If there's a binding, add to its references too
        if let Some(binding_id) = self.get(scope_id, &name) {
            self.bindings[binding_id.index()]
                .references
                .push(reference);
        } else {
            // Global reference - add to conflicts
            self.conflicts.insert(name);
        }
    }

    /// Finds the scope that owns a binding.
    pub fn owner(&self, scope_id: ScopeId, name: &str) -> Option<ScopeId> {
        let mut current = Some(scope_id);
        while let Some(id) = current {
            let scope = &self.scopes[id.index()];
            if scope.declarations.contains_key(name) {
                return Some(id);
            }
            current = scope.parent;
        }
        None
    }

    /// Generates a unique name that doesn't conflict with existing names.
    pub fn generate(&mut self, scope_id: ScopeId, preferred: &str) -> String {
        // Find the nearest non-porous scope
        let target_scope_id = self.find_var_scope(scope_id);

        let mut name = preferred
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '_' || c == '$' { c } else { '_' })
            .collect::<String>();

        // Ensure it doesn't start with a digit
        if name.chars().next().map_or(false, |c| c.is_ascii_digit()) {
            name.insert(0, '_');
        }

        let base = name.clone();
        let mut counter = 1;

        let scope = &self.scopes[target_scope_id.index()];
        while scope.references.contains_key(&name)
            || scope.declarations.contains_key(&name)
            || self.conflicts.contains(&name)
            || is_reserved(&name)
        {
            name = format!("{}_{}", base, counter);
            counter += 1;
        }

        // Reserve the name
        self.scopes[target_scope_id.index()]
            .references
            .entry(name.clone())
            .or_default();
        self.conflicts.insert(name.clone());

        name
    }
}

impl Default for ScopeTree {
    fn default() -> Self {
        Self::new()
    }
}

impl ScopeTree {
    /// Returns an iterator over all scopes.
    pub fn iter_scopes(&self) -> impl Iterator<Item = (ScopeId, &Scope)> {
        self.scopes
            .iter()
            .enumerate()
            .map(|(i, scope)| (ScopeId::new(i as u32), scope))
    }
}
