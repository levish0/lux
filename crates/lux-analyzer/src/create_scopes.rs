//! First pass: Create the scope tree from the AST.
//!
//! This module traverses the AST and builds the scope tree by:
//! - Creating scopes for functions, blocks, and template constructs
//! - Declaring bindings for variables, functions, imports, etc.
//! - Recording references to identifiers
//! - Tracking assignments and updates

use oxc_ast::ast::{BindingPattern, FormalParameters, Statement, VariableDeclarationKind};
use oxc_ast_visit::{walk, Visit};
use oxc_span::Span;
use oxc_syntax::scope::ScopeFlags;

use crate::scope::{BindingKind, DeclarationKind, ScopeId, ScopeTree};
use crate::visitor::{self, SvelteVisitor};
use lux_ast::blocks::{AwaitBlock, EachBlock, SnippetBlock};
use lux_ast::elements::{
    Component, RegularElement, SlotElement, SvelteComponent, SvelteElement, SvelteFragment,
    SvelteSelf,
};
use lux_ast::root::{Fragment, Root, Script};

/// Result of the first pass scope creation.
pub struct ScopeCreationResult {
    /// The scope tree containing all scopes and bindings
    pub scopes: ScopeTree,
    /// Whether the AST contains a top-level await (outside of functions)
    pub has_await: bool,
}

/// Creates the scope tree from the AST.
pub fn create_scopes(root: &Root<'_>) -> ScopeCreationResult {
    let mut creator = ScopeCreator::new();
    creator.visit_root(root);

    ScopeCreationResult {
        scopes: creator.scopes,
        has_await: creator.has_await,
    }
}

/// Visitor that builds the scope tree.
struct ScopeCreator {
    scopes: ScopeTree,
    current_scope: ScopeId,
    has_await: bool,
    /// Track function depth to detect top-level awaits
    function_depth: u32,
    /// Whether reactive declarations ($:) are allowed in the current context
    allow_reactive_declarations: bool,
}

impl ScopeCreator {
    fn new() -> Self {
        let scopes = ScopeTree::new();
        Self {
            current_scope: scopes.root_scope_id(),
            scopes,
            has_await: false,
            function_depth: 0,
            allow_reactive_declarations: false,
        }
    }

    /// Creates a child scope and returns the parent scope id for restoration.
    fn enter_scope(&mut self, porous: bool) -> ScopeId {
        let parent = self.current_scope;
        self.current_scope = self.scopes.create_child_scope(parent, porous);
        parent
    }

    /// Restores to the parent scope.
    fn exit_scope(&mut self, parent: ScopeId) {
        self.current_scope = parent;
    }

    /// Declares a binding in the current scope.
    fn declare(&mut self, name: &str, span: Span, kind: BindingKind, decl_kind: DeclarationKind) {
        self.scopes
            .declare(self.current_scope, name.to_string(), span, kind, decl_kind);
    }

    /// Records a reference to an identifier in the current scope.
    #[allow(dead_code)]
    fn reference(&mut self, name: &str, span: Span) {
        self.scopes
            .reference(self.current_scope, name.to_string(), span);
    }

    /// Declares all identifiers in a binding pattern.
    fn declare_binding_pattern(
        &mut self,
        pattern: &BindingPattern<'_>,
        kind: BindingKind,
        decl_kind: DeclarationKind,
    ) {
        match pattern {
            BindingPattern::BindingIdentifier(id) => {
                self.declare(&id.name, id.span, kind, decl_kind);
            }
            BindingPattern::ObjectPattern(obj) => {
                for prop in &obj.properties {
                    self.declare_binding_pattern(&prop.value, kind, decl_kind);
                }
                if let Some(ref rest) = obj.rest {
                    self.declare_binding_pattern(&rest.argument, kind, decl_kind);
                }
            }
            BindingPattern::ArrayPattern(arr) => {
                for elem in arr.elements.iter().flatten() {
                    self.declare_binding_pattern(elem, kind, decl_kind);
                }
                if let Some(ref rest) = arr.rest {
                    self.declare_binding_pattern(&rest.argument, kind, decl_kind);
                }
            }
            BindingPattern::AssignmentPattern(assign) => {
                self.declare_binding_pattern(&assign.left, kind, decl_kind);
            }
        }
    }

    fn declare_params(&mut self, params: &FormalParameters<'_>) {
        for param in &params.items {
            self.declare_binding_pattern(&param.pattern, BindingKind::Normal, DeclarationKind::Param);
        }
        if let Some(ref rest) = params.rest {
            self.declare_binding_pattern(
                &rest.rest.argument,
                BindingKind::Normal,
                DeclarationKind::RestParam,
            );
        }
    }
}

impl<'a> SvelteVisitor<'a> for ScopeCreator {
    fn visit_root(&mut self, node: &Root<'a>) {
        // Module script: no reactive declarations allowed
        if let Some(ref module) = node.module {
            self.allow_reactive_declarations = false;
            self.visit_script(module);
        }

        // Instance script: reactive declarations allowed
        if let Some(ref instance) = node.instance {
            self.allow_reactive_declarations = true;
            self.visit_script(instance);
        }

        // Template: create child scope from instance
        self.allow_reactive_declarations = false;
        let parent = self.enter_scope(false);
        self.visit_fragment(&node.fragment);
        self.exit_scope(parent);
    }

    fn visit_script(&mut self, node: &Script<'a>) {
        // Script content is a JS Program - we need to visit it with oxc's visitor
        let parent = self.enter_scope(false);

        // Visit the JS AST
        let mut js_visitor = JsVisitor {
            creator: self,
            in_function: false,
        };
        js_visitor.visit_program(&node.content);

        self.exit_scope(parent);
    }

    fn visit_fragment(&mut self, node: &Fragment<'a>) {
        // Fragment creates a transparent scope (porous)
        let parent = self.enter_scope(true);
        visitor::walk_fragment(self, node);
        self.exit_scope(parent);
    }

    // Block constructs
    fn visit_each_block(&mut self, node: &EachBlock<'a>) {
        // Visit expression in parent scope
        // TODO: visit expression

        // Create scope for context and children
        let parent = self.enter_scope(false);

        // Declare context binding (FormalParameter -> BindingPattern)
        if let Some(ref context) = node.context {
            self.declare_binding_pattern(&context.pattern, BindingKind::Each, DeclarationKind::Const);
        }

        // Declare index if present
        if let Some(ref index) = node.index {
            // Check if keyed
            let kind = if node.key.is_some() {
                BindingKind::Template
            } else {
                BindingKind::Static
            };
            self.declare(
                index,
                Span::new(0, 0), // TODO: get proper span
                kind,
                DeclarationKind::Const,
            );
        }

        // Visit body and fallback
        self.visit_fragment(&node.body);
        if let Some(ref fallback) = node.fallback {
            self.visit_fragment(fallback);
        }

        self.exit_scope(parent);
    }

    fn visit_await_block(&mut self, node: &AwaitBlock<'a>) {
        // Visit expression in current scope
        // TODO: visit expression

        // Pending block
        if let Some(ref pending) = node.pending {
            self.visit_fragment(pending);
        }

        // Then block with value binding
        if let Some(ref then) = node.then {
            let parent = self.enter_scope(false);
            if let Some(ref value) = node.value {
                self.declare_binding_pattern(&value.pattern, BindingKind::Template, DeclarationKind::Const);
            }
            self.visit_fragment(then);
            self.exit_scope(parent);
        }

        // Catch block with error binding
        if let Some(ref catch) = node.catch {
            let parent = self.enter_scope(false);
            if let Some(ref error) = node.error {
                self.declare_binding_pattern(&error.pattern, BindingKind::Template, DeclarationKind::Const);
            }
            self.visit_fragment(catch);
            self.exit_scope(parent);
        }
    }

    fn visit_snippet_block(&mut self, node: &SnippetBlock<'a>) {
        // Declare snippet name in current scope (expression is Identifier)
        if let oxc_ast::ast::Expression::Identifier(ref id) = node.expression {
            self.declare(
                &id.name,
                id.span,
                BindingKind::Normal,
                DeclarationKind::Function,
            );
        }

        // Create child scope for parameters and body
        let parent = self.enter_scope(false);

        for param in &node.parameters {
            self.declare_binding_pattern(&param.pattern, BindingKind::Snippet, DeclarationKind::Let);
        }

        self.visit_fragment(&node.body);
        self.exit_scope(parent);
    }

    // Elements create child scopes
    fn visit_regular_element(&mut self, node: &RegularElement<'a>) {
        let parent = self.enter_scope(false);
        visitor::walk_regular_element(self, node);
        self.exit_scope(parent);
    }

    // Component scopes are more complex (multiple slots)
    fn visit_component(&mut self, node: &Component<'a>) {
        // TODO: handle multiple slot scopes
        let parent = self.enter_scope(false);
        visitor::walk_component(self, node);
        self.exit_scope(parent);
    }

    fn visit_svelte_element(&mut self, node: &SvelteElement<'a>) {
        let parent = self.enter_scope(false);
        visitor::walk_svelte_element(self, node);
        self.exit_scope(parent);
    }

    fn visit_svelte_component(&mut self, node: &SvelteComponent<'a>) {
        let parent = self.enter_scope(false);
        visitor::walk_svelte_component(self, node);
        self.exit_scope(parent);
    }

    fn visit_svelte_self(&mut self, node: &SvelteSelf<'a>) {
        let parent = self.enter_scope(false);
        visitor::walk_svelte_self(self, node);
        self.exit_scope(parent);
    }

    fn visit_slot_element(&mut self, node: &SlotElement<'a>) {
        let parent = self.enter_scope(false);
        visitor::walk_slot_element(self, node);
        self.exit_scope(parent);
    }

    fn visit_svelte_fragment(&mut self, node: &SvelteFragment<'a>) {
        let parent = self.enter_scope(false);
        visitor::walk_svelte_fragment(self, node);
        self.exit_scope(parent);
    }

    fn visit_let_directive(&mut self, node: &lux_ast::attributes::LetDirective<'a>) {
        // let: declarations create bindings in the current scope
        // expression is an Expression (could be Identifier for destructuring)
        if let Some(ref expression) = node.expression {
            // Try to extract identifier from expression
            if let oxc_ast::ast::Expression::Identifier(id) = expression {
                self.declare(
                    &id.name,
                    id.span,
                    BindingKind::Template,
                    DeclarationKind::Const,
                );
            }
            // TODO: handle destructuring patterns
        } else {
            self.declare(
                node.name,
                Span::new(node.span.start as u32, node.span.end as u32),
                BindingKind::Template,
                DeclarationKind::Const,
            );
        }
    }
}

// ============================================================================
// JavaScript AST visitor
// ============================================================================

/// Visitor for JavaScript AST nodes within scripts.
struct JsVisitor<'b> {
    creator: &'b mut ScopeCreator,
    in_function: bool,
}

impl<'a> Visit<'a> for JsVisitor<'_> {
    fn visit_identifier_reference(&mut self, id: &oxc_ast::ast::IdentifierReference<'a>) {
        self.creator
            .scopes
            .reference(self.creator.current_scope, id.name.to_string(), id.span);
    }

    fn visit_await_expression(&mut self, expr: &oxc_ast::ast::AwaitExpression<'a>) {
        // Track top-level awaits
        if !self.in_function {
            self.creator.has_await = true;
        }
        walk::walk_await_expression(self, expr);
    }

    fn visit_block_statement(&mut self, block: &oxc_ast::ast::BlockStatement<'a>) {
        let parent = self.creator.enter_scope(true);
        walk::walk_block_statement(self, block);
        self.creator.exit_scope(parent);
    }

    fn visit_variable_declaration(&mut self, decl: &oxc_ast::ast::VariableDeclaration<'a>) {
        let decl_kind = match decl.kind {
            VariableDeclarationKind::Var => DeclarationKind::Var,
            VariableDeclarationKind::Let => DeclarationKind::Let,
            VariableDeclarationKind::Const => DeclarationKind::Const,
            VariableDeclarationKind::Using => DeclarationKind::Using,
            VariableDeclarationKind::AwaitUsing => DeclarationKind::AwaitUsing,
        };

        for declarator in &decl.declarations {
            self.creator
                .declare_binding_pattern(&declarator.id, BindingKind::Normal, decl_kind);

            // Visit initializer
            if let Some(ref init) = declarator.init {
                self.visit_expression(init);
            }
        }
    }

    fn visit_for_statement(&mut self, stmt: &oxc_ast::ast::ForStatement<'a>) {
        let parent = self.creator.enter_scope(true);
        walk::walk_for_statement(self, stmt);
        self.creator.exit_scope(parent);
    }

    fn visit_for_in_statement(&mut self, stmt: &oxc_ast::ast::ForInStatement<'a>) {
        let parent = self.creator.enter_scope(true);
        walk::walk_for_in_statement(self, stmt);
        self.creator.exit_scope(parent);
    }

    fn visit_for_of_statement(&mut self, stmt: &oxc_ast::ast::ForOfStatement<'a>) {
        let parent = self.creator.enter_scope(true);
        walk::walk_for_of_statement(self, stmt);
        self.creator.exit_scope(parent);
    }

    fn visit_switch_statement(&mut self, stmt: &oxc_ast::ast::SwitchStatement<'a>) {
        let parent = self.creator.enter_scope(true);
        walk::walk_switch_statement(self, stmt);
        self.creator.exit_scope(parent);
    }

    fn visit_labeled_statement(&mut self, stmt: &oxc_ast::ast::LabeledStatement<'a>) {
        // Handle $: reactive declarations
        if self.creator.allow_reactive_declarations
            && self.creator.function_depth == 0
            && stmt.label.name == "$"
        {
            let parent = self.creator.enter_scope(false);

            // Check for implicit declarations: $: a = b * 2
            if let Statement::ExpressionStatement(expr_stmt) = &stmt.body {
                if let oxc_ast::ast::Expression::AssignmentExpression(assign) =
                    &expr_stmt.expression
                {
                    // Extract identifiers from left side as potential implicit declarations
                    self.check_implicit_declaration(&assign.left);
                }
            }

            walk::walk_labeled_statement(self, stmt);
            self.creator.exit_scope(parent);
        } else {
            walk::walk_labeled_statement(self, stmt);
        }
    }

    fn visit_catch_clause(&mut self, clause: &oxc_ast::ast::CatchClause<'a>) {
        let parent = self.creator.enter_scope(true);
        if let Some(ref param) = clause.param {
            self.creator
                .declare_binding_pattern(&param.pattern, BindingKind::Normal, DeclarationKind::Let);
        }
        walk::walk_catch_clause(self, clause);
        self.creator.exit_scope(parent);
    }

    fn visit_function(&mut self, func: &oxc_ast::ast::Function<'a>, _flags: ScopeFlags) {
        // Declare function name in current scope (for declarations)
        if let Some(ref id) = func.id {
            self.creator.declare(
                &id.name,
                id.span,
                BindingKind::Normal,
                DeclarationKind::Function,
            );
        }

        // Create child scope for function body
        let parent = self.creator.enter_scope(false);
        self.creator.function_depth += 1;
        let was_in_function = self.in_function;
        self.in_function = true;

        // Declare parameters
        self.creator.declare_params(&func.params);

        // Visit body
        if let Some(ref body) = func.body {
            walk::walk_function_body(self, body);
        }

        self.in_function = was_in_function;
        self.creator.function_depth -= 1;
        self.creator.exit_scope(parent);
    }

    fn visit_arrow_function_expression(
        &mut self,
        func: &oxc_ast::ast::ArrowFunctionExpression<'a>,
    ) {
        let parent = self.creator.enter_scope(false);
        self.creator.function_depth += 1;
        let was_in_function = self.in_function;
        self.in_function = true;

        // Declare parameters
        self.creator.declare_params(&func.params);

        // Visit body - just continue walking
        walk::walk_arrow_function_expression(self, func);

        self.in_function = was_in_function;
        self.creator.function_depth -= 1;
        self.creator.exit_scope(parent);
    }

    fn visit_class(&mut self, class: &oxc_ast::ast::Class<'a>) {
        if let Some(ref id) = class.id {
            self.creator
                .declare(&id.name, id.span, BindingKind::Normal, DeclarationKind::Let);
        }
        walk::walk_class(self, class);
    }

    fn visit_import_declaration(&mut self, import: &oxc_ast::ast::ImportDeclaration<'a>) {
        if let Some(ref specifiers) = import.specifiers {
            for specifier in specifiers {
                match specifier {
                    oxc_ast::ast::ImportDeclarationSpecifier::ImportSpecifier(s) => {
                        self.creator.declare(
                            &s.local.name,
                            s.local.span,
                            BindingKind::Normal,
                            DeclarationKind::Import,
                        );
                    }
                    oxc_ast::ast::ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                        self.creator.declare(
                            &s.local.name,
                            s.local.span,
                            BindingKind::Normal,
                            DeclarationKind::Import,
                        );
                    }
                    oxc_ast::ast::ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                        self.creator.declare(
                            &s.local.name,
                            s.local.span,
                            BindingKind::Normal,
                            DeclarationKind::Import,
                        );
                    }
                }
            }
        }
    }
}

impl JsVisitor<'_> {
    fn check_implicit_declaration(&mut self, target: &oxc_ast::ast::AssignmentTarget<'_>) {
        match target {
            oxc_ast::ast::AssignmentTarget::AssignmentTargetIdentifier(id) => {
                // Check if already declared
                if self
                    .creator
                    .scopes
                    .get(self.creator.current_scope, &id.name)
                    .is_none()
                    && !id.name.starts_with('$')
                {
                    // Implicit declaration
                    self.creator.declare(
                        &id.name,
                        id.span,
                        BindingKind::LegacyReactive,
                        DeclarationKind::Let,
                    );
                }
            }
            oxc_ast::ast::AssignmentTarget::ArrayAssignmentTarget(arr) => {
                for elem in arr.elements.iter().flatten() {
                    self.check_implicit_declaration_target(elem);
                }
            }
            oxc_ast::ast::AssignmentTarget::ObjectAssignmentTarget(obj) => {
                for prop in &obj.properties {
                    match prop {
                        oxc_ast::ast::AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(id) => {
                            // Simple case: { foo } = obj
                            if self
                                .creator
                                .scopes
                                .get(self.creator.current_scope, &id.binding.name)
                                .is_none()
                                && !id.binding.name.starts_with('$')
                            {
                                self.creator.declare(
                                    &id.binding.name,
                                    id.binding.span,
                                    BindingKind::LegacyReactive,
                                    DeclarationKind::Let,
                                );
                            }
                        }
                        oxc_ast::ast::AssignmentTargetProperty::AssignmentTargetPropertyProperty(p) => {
                            // Renamed case: { foo: bar } = obj
                            self.check_implicit_declaration_target(&p.binding);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn check_implicit_declaration_target(
        &mut self,
        target: &oxc_ast::ast::AssignmentTargetMaybeDefault<'_>,
    ) {
        match target {
            oxc_ast::ast::AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(t) => {
                self.check_implicit_declaration(&t.binding);
            }
            _ => {
                if let Some(t) = target.as_assignment_target() {
                    self.check_implicit_declaration(t);
                }
            }
        }
    }
}
