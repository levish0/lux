//! ScopeCreator and SvelteVisitor implementation.

use oxc_ast::ast::{BindingPattern, FormalParameters};
use oxc_ast_visit::Visit;
use oxc_span::{GetSpan, Span};

use super::js::JsVisitor;
use super::{Assignment, Update};
use crate::scope::{BindingKind, DeclarationKind, ScopeId, ScopeTree};
use crate::visitor::{self, SvelteVisitor};
use lux_ast::blocks::{AwaitBlock, EachBlock, SnippetBlock};
use lux_ast::elements::{
    Component, RegularElement, SlotElement, SvelteComponent, SvelteElement, SvelteFragment,
    SvelteSelf,
};
use lux_ast::root::{Fragment, Root, Script};

/// Visitor that builds the scope tree.
pub(crate) struct ScopeCreator {
    pub scopes: ScopeTree,
    pub current_scope: ScopeId,
    pub has_await: bool,
    /// Track function depth to detect top-level awaits
    pub function_depth: u32,
    /// Whether reactive declarations ($:) are allowed in the current context
    pub allow_reactive_declarations: bool,
    /// Collected updates to process at the end
    updates: Vec<Update>,
}

impl ScopeCreator {
    pub fn new() -> Self {
        let scopes = ScopeTree::new();
        Self {
            current_scope: scopes.root_scope_id(),
            scopes,
            has_await: false,
            function_depth: 0,
            allow_reactive_declarations: false,
            updates: Vec::new(),
        }
    }

    /// Process all collected updates to mark bindings as mutated or reassigned.
    pub fn process_updates(&mut self) {
        // Take ownership of updates to avoid borrow issues
        let updates = std::mem::take(&mut self.updates);

        for update in updates {
            // Look up the binding
            if let Some(binding_id) = self.scopes.get(update.scope_id, &update.name) {
                let binding = self.scopes.get_binding_mut(binding_id);

                // Don't mark as reassigned/mutated if assigning to self (initial assignment)
                if binding.node_span != update.left_span {
                    if update.is_direct {
                        binding.reassigned = true;
                        binding.assignments.push(Assignment {
                            value_span: update.value_span,
                            scope: update.scope_id,
                        });
                    } else {
                        binding.mutated = true;
                    }
                }
            }
        }
    }

    /// Records an update (assignment or mutation) for later processing.
    pub fn record_update(&mut self, name: String, left_span: Span, value_span: Span, is_direct: bool) {
        self.updates.push(Update {
            scope_id: self.current_scope,
            name,
            left_span,
            value_span,
            is_direct,
        });
    }

    /// Creates a child scope and returns the parent scope id for restoration.
    pub fn enter_scope(&mut self, porous: bool) -> ScopeId {
        let parent = self.current_scope;
        self.current_scope = self.scopes.create_child_scope(parent, porous);
        parent
    }

    /// Restores to the parent scope.
    pub fn exit_scope(&mut self, parent: ScopeId) {
        self.current_scope = parent;
    }

    /// Declares a binding in the current scope.
    pub fn declare(&mut self, name: &str, span: Span, kind: BindingKind, decl_kind: DeclarationKind) {
        self.scopes
            .declare(self.current_scope, name.to_string(), span, kind, decl_kind);
    }

    /// Records a reference to an identifier in the current scope.
    #[allow(dead_code)]
    pub fn reference(&mut self, name: &str, span: Span) {
        self.scopes
            .reference(self.current_scope, name.to_string(), span);
    }

    /// Declares all identifiers in a binding pattern.
    pub fn declare_binding_pattern(
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

    pub fn declare_params(&mut self, params: &FormalParameters<'_>) {
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

    /// Declares bindings from a let: directive expression (handles destructuring).
    fn declare_let_directive_pattern(&mut self, expression: &oxc_ast::ast::Expression<'_>) {
        match expression {
            oxc_ast::ast::Expression::ObjectExpression(obj) => {
                for prop in &obj.properties {
                    if let oxc_ast::ast::ObjectPropertyKind::ObjectProperty(p) = prop {
                        if let oxc_ast::ast::Expression::Identifier(id) = &p.value {
                            self.declare(
                                &id.name,
                                id.span,
                                BindingKind::Template,
                                DeclarationKind::Const,
                            );
                        }
                    }
                }
            }
            oxc_ast::ast::Expression::ArrayExpression(arr) => {
                for elem in &arr.elements {
                    if let oxc_ast::ast::ArrayExpressionElement::Identifier(id) = elem {
                        self.declare(
                            &id.name,
                            id.span,
                            BindingKind::Template,
                            DeclarationKind::Const,
                        );
                    }
                }
            }
            _ => {}
        }
    }

    /// Records an update for a bind: directive expression.
    fn record_bind_update(&mut self, expression: &oxc_ast::ast::Expression<'_>) {
        match expression {
            oxc_ast::ast::Expression::Identifier(id) => {
                self.record_update(id.name.to_string(), id.span, id.span, true);
            }
            oxc_ast::ast::Expression::StaticMemberExpression(member) => {
                if let Some(name) = Self::get_expression_object_name(&member.object) {
                    self.record_update(name, member.object.span(), member.span(), false);
                }
            }
            oxc_ast::ast::Expression::ComputedMemberExpression(member) => {
                if let Some(name) = Self::get_expression_object_name(&member.object) {
                    self.record_update(name, member.object.span(), member.span(), false);
                }
            }
            _ => {}
        }
    }

    /// Gets the root object name from an expression.
    pub fn get_expression_object_name(expr: &oxc_ast::ast::Expression<'_>) -> Option<String> {
        match expr {
            oxc_ast::ast::Expression::Identifier(id) => Some(id.name.to_string()),
            oxc_ast::ast::Expression::StaticMemberExpression(member) => {
                Self::get_expression_object_name(&member.object)
            }
            oxc_ast::ast::Expression::ComputedMemberExpression(member) => {
                Self::get_expression_object_name(&member.object)
            }
            _ => None,
        }
    }
}

impl<'a> SvelteVisitor<'a> for ScopeCreator {
    fn visit_root(&mut self, node: &Root<'a>) {
        if let Some(ref module) = node.module {
            self.allow_reactive_declarations = false;
            self.visit_script(module);
        }

        if let Some(ref instance) = node.instance {
            self.allow_reactive_declarations = true;
            self.visit_script(instance);
        }

        self.allow_reactive_declarations = false;
        let parent = self.enter_scope(false);
        self.visit_fragment(&node.fragment);
        self.exit_scope(parent);
    }

    fn visit_script(&mut self, node: &Script<'a>) {
        let parent = self.enter_scope(false);

        let mut js_visitor = JsVisitor::new(self);
        js_visitor.visit_program(&node.content);

        self.exit_scope(parent);
    }

    fn visit_fragment(&mut self, node: &Fragment<'a>) {
        let parent = self.enter_scope(true);
        visitor::walk_fragment(self, node);
        self.exit_scope(parent);
    }

    fn visit_each_block(&mut self, node: &EachBlock<'a>) {
        let parent = self.enter_scope(false);

        if let Some(ref context) = node.context {
            self.declare_binding_pattern(&context.pattern, BindingKind::Each, DeclarationKind::Const);
        }

        if let Some(ref index) = node.index {
            let kind = if node.key.is_some() {
                BindingKind::Template
            } else {
                BindingKind::Static
            };
            self.declare(index, Span::new(0, 0), kind, DeclarationKind::Const);
        }

        self.visit_fragment(&node.body);
        if let Some(ref fallback) = node.fallback {
            self.visit_fragment(fallback);
        }

        self.exit_scope(parent);
    }

    fn visit_await_block(&mut self, node: &AwaitBlock<'a>) {
        if let Some(ref pending) = node.pending {
            self.visit_fragment(pending);
        }

        if let Some(ref then) = node.then {
            let parent = self.enter_scope(false);
            if let Some(ref value) = node.value {
                self.declare_binding_pattern(&value.pattern, BindingKind::Template, DeclarationKind::Const);
            }
            self.visit_fragment(then);
            self.exit_scope(parent);
        }

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
        if let oxc_ast::ast::Expression::Identifier(ref id) = node.expression {
            self.declare(&id.name, id.span, BindingKind::Normal, DeclarationKind::Function);
        }

        let parent = self.enter_scope(false);

        for param in &node.parameters {
            self.declare_binding_pattern(&param.pattern, BindingKind::Snippet, DeclarationKind::Let);
        }

        self.visit_fragment(&node.body);
        self.exit_scope(parent);
    }

    fn visit_regular_element(&mut self, node: &RegularElement<'a>) {
        let parent = self.enter_scope(false);
        visitor::walk_regular_element(self, node);
        self.exit_scope(parent);
    }

    fn visit_component(&mut self, node: &Component<'a>) {
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

    fn visit_bind_directive(&mut self, node: &lux_ast::attributes::BindDirective<'a>) {
        if !matches!(node.expression, oxc_ast::ast::Expression::SequenceExpression(_)) {
            self.record_bind_update(&node.expression);
        }
        visitor::walk_bind_directive(self, node);
    }

    fn visit_let_directive(&mut self, node: &lux_ast::attributes::LetDirective<'a>) {
        if let Some(ref expression) = node.expression {
            if let oxc_ast::ast::Expression::Identifier(id) = expression {
                self.declare(&id.name, id.span, BindingKind::Template, DeclarationKind::Const);
            }
            self.declare_let_directive_pattern(expression);
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
