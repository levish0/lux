//! JsVisitor for JavaScript AST traversal within scripts.

use oxc_ast::ast::{Statement, VariableDeclarationKind};
use oxc_ast_visit::{walk, Visit};
use oxc_span::{GetSpan, Span};
use oxc_syntax::scope::ScopeFlags;

use super::svelte::ScopeCreator;
use crate::scope::{BindingKind, DeclarationKind};

/// Visitor for JavaScript AST nodes within scripts.
pub(crate) struct JsVisitor<'b> {
    pub creator: &'b mut ScopeCreator,
    pub in_function: bool,
}

impl<'b> JsVisitor<'b> {
    pub fn new(creator: &'b mut ScopeCreator) -> Self {
        Self {
            creator,
            in_function: false,
        }
    }
}

impl<'a> Visit<'a> for JsVisitor<'_> {
    fn visit_identifier_reference(&mut self, id: &oxc_ast::ast::IdentifierReference<'a>) {
        self.creator
            .scopes
            .reference(self.creator.current_scope, id.name.to_string(), id.span);
    }

    fn visit_update_expression(&mut self, expr: &oxc_ast::ast::UpdateExpression<'a>) {
        self.record_simple_expression_update(&expr.argument, expr.argument.span());
        walk::walk_update_expression(self, expr);
    }

    fn visit_assignment_expression(&mut self, expr: &oxc_ast::ast::AssignmentExpression<'a>) {
        self.record_assignment_target_update(&expr.left, expr.right.span());
        walk::walk_assignment_expression(self, expr);
    }

    fn visit_await_expression(&mut self, expr: &oxc_ast::ast::AwaitExpression<'a>) {
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
        if self.creator.allow_reactive_declarations
            && self.creator.function_depth == 0
            && stmt.label.name == "$"
        {
            let parent = self.creator.enter_scope(false);

            if let Statement::ExpressionStatement(expr_stmt) = &stmt.body {
                if let oxc_ast::ast::Expression::AssignmentExpression(assign) = &expr_stmt.expression
                {
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
        if let Some(ref id) = func.id {
            self.creator.declare(
                &id.name,
                id.span,
                BindingKind::Normal,
                DeclarationKind::Function,
            );
        }

        let parent = self.creator.enter_scope(false);
        self.creator.function_depth += 1;
        let was_in_function = self.in_function;
        self.in_function = true;

        self.creator.declare_params(&func.params);

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

        self.creator.declare_params(&func.params);
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

// Helper methods for JsVisitor
impl JsVisitor<'_> {
    /// Records an update for an assignment target.
    fn record_assignment_target_update(
        &mut self,
        target: &oxc_ast::ast::AssignmentTarget<'_>,
        value_span: Span,
    ) {
        match target {
            oxc_ast::ast::AssignmentTarget::AssignmentTargetIdentifier(id) => {
                self.creator
                    .record_update(id.name.to_string(), id.span, value_span, true);
            }
            oxc_ast::ast::AssignmentTarget::StaticMemberExpression(member) => {
                if let Some(name) = self.get_object_name(&member.object) {
                    self.creator
                        .record_update(name, member.object.span(), value_span, false);
                }
            }
            oxc_ast::ast::AssignmentTarget::ComputedMemberExpression(member) => {
                if let Some(name) = self.get_object_name(&member.object) {
                    self.creator
                        .record_update(name, member.object.span(), value_span, false);
                }
            }
            oxc_ast::ast::AssignmentTarget::ArrayAssignmentTarget(arr) => {
                for elem in arr.elements.iter().flatten() {
                    self.record_assignment_target_maybe_default_update(elem, value_span);
                }
            }
            oxc_ast::ast::AssignmentTarget::ObjectAssignmentTarget(obj) => {
                for prop in &obj.properties {
                    match prop {
                        oxc_ast::ast::AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(id) => {
                            self.creator.record_update(
                                id.binding.name.to_string(),
                                id.binding.span,
                                value_span,
                                true,
                            );
                        }
                        oxc_ast::ast::AssignmentTargetProperty::AssignmentTargetPropertyProperty(p) => {
                            self.record_assignment_target_maybe_default_update(&p.binding, value_span);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn record_assignment_target_maybe_default_update(
        &mut self,
        target: &oxc_ast::ast::AssignmentTargetMaybeDefault<'_>,
        value_span: Span,
    ) {
        match target {
            oxc_ast::ast::AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(t) => {
                self.record_assignment_target_update(&t.binding, value_span);
            }
            _ => {
                if let Some(t) = target.as_assignment_target() {
                    self.record_assignment_target_update(t, value_span);
                }
            }
        }
    }

    /// Records an update for a simple expression (++x, x++).
    fn record_simple_expression_update(
        &mut self,
        target: &oxc_ast::ast::SimpleAssignmentTarget<'_>,
        value_span: Span,
    ) {
        match target {
            oxc_ast::ast::SimpleAssignmentTarget::AssignmentTargetIdentifier(id) => {
                self.creator
                    .record_update(id.name.to_string(), id.span, value_span, true);
            }
            oxc_ast::ast::SimpleAssignmentTarget::StaticMemberExpression(member) => {
                if let Some(name) = self.get_object_name(&member.object) {
                    self.creator
                        .record_update(name, member.object.span(), value_span, false);
                }
            }
            oxc_ast::ast::SimpleAssignmentTarget::ComputedMemberExpression(member) => {
                if let Some(name) = self.get_object_name(&member.object) {
                    self.creator
                        .record_update(name, member.object.span(), value_span, false);
                }
            }
            _ => {}
        }
    }

    /// Gets the root object name from a member expression chain.
    fn get_object_name(&self, expr: &oxc_ast::ast::Expression<'_>) -> Option<String> {
        match expr {
            oxc_ast::ast::Expression::Identifier(id) => Some(id.name.to_string()),
            oxc_ast::ast::Expression::StaticMemberExpression(member) => {
                self.get_object_name(&member.object)
            }
            oxc_ast::ast::Expression::ComputedMemberExpression(member) => {
                self.get_object_name(&member.object)
            }
            _ => None,
        }
    }

    fn check_implicit_declaration(&mut self, target: &oxc_ast::ast::AssignmentTarget<'_>) {
        match target {
            oxc_ast::ast::AssignmentTarget::AssignmentTargetIdentifier(id) => {
                if self
                    .creator
                    .scopes
                    .get(self.creator.current_scope, &id.name)
                    .is_none()
                    && !id.name.starts_with('$')
                {
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
