//! Analysis visitor that walks both Svelte and JS ASTs.
//!
//! This visitor performs the actual analysis work by traversing the AST
//! and collecting metadata, validating, and updating bindings.

use oxc_ast::ast::{
    AssignmentExpression, AwaitExpression, BindingPattern, CallExpression, Declaration, Expression,
    ExportNamedDeclaration, ExpressionStatement, IdentifierReference, ImportDeclaration,
    ImportDeclarationSpecifier, LabeledStatement, MemberExpression, ModuleExportName, NewExpression,
    UpdateExpression, VariableDeclarationKind, VariableDeclarator,
};
use oxc_ast_visit::{walk, Visit};
use oxc_span::{GetSpan, Span};
use oxc_syntax::scope::ScopeFlags;

use super::utils::{
    extract_identifiers_from_pattern, get_rune, validate_identifier_name, IdentifierNameError,
};
use crate::scope::BindingKind;

use lux_ast::attributes::{
    AnimateDirective, BindDirective, ClassDirective, LetDirective, OnDirective, SpreadAttribute,
    StyleDirective, TransitionDirective, UseDirective,
};
use lux_ast::blocks::{AwaitBlock, EachBlock, IfBlock, KeyBlock};
use lux_ast::elements::{
    Component, RegularElement, SlotElement, SvelteBody, SvelteBoundary, SvelteComponent,
    SvelteDocument, SvelteElement, SvelteFragment, SvelteHead, SvelteSelf, SvelteWindow,
    TitleElement,
};
use lux_ast::root::{Fragment, Root, Script};
use lux_ast::tags::{ConstTag, DebugTag, ExpressionTag, HtmlTag, RenderTag};

use super::analysis::ComponentAnalysis;
use super::state::{AnalysisState, AstType};
use super::visitors::shared::validate_element;
use super::visitors::{
    visit_animate_directive, visit_await_block, visit_bind_directive, visit_call_expression,
    visit_class_directive, visit_component, visit_const_tag, visit_debug_tag, visit_each_block,
    visit_expression_tag, visit_html_tag, visit_identifier, visit_if_block, visit_key_block,
    visit_let_directive, visit_on_directive, visit_regular_element, visit_render_tag,
    visit_slot_element, visit_snippet_block, visit_spread_attribute, visit_style_directive,
    visit_svelte_body, visit_svelte_boundary, visit_svelte_component, visit_svelte_document,
    visit_svelte_element, visit_svelte_fragment, visit_svelte_head, visit_svelte_self,
    visit_svelte_window, visit_text, visit_title_element, visit_transition_directive,
    visit_use_directive,
};
use crate::scope::ScopeId;
use crate::visitor::{self, SvelteVisitor};

/// Main analysis visitor that walks the entire AST.
pub struct AnalysisVisitor<'s, 'a, 'b> {
    pub state: AnalysisState<'s, 'b>,
    /// Path of ancestor nodes (for context)
    pub path: Vec<NodeKind<'a>>,
}

/// Node kind for path tracking.
/// Matches the element types from lux_ast for proper parent type checking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind<'a> {
    // Root nodes
    Root,
    Script,
    Fragment,

    // Regular elements
    RegularElement(&'a str),
    TitleElement,

    // Components
    Component(&'a str),
    SvelteComponent,
    SvelteSelf,

    // Svelte special elements
    SvelteElement,
    SvelteHead,
    SvelteBody,
    SvelteWindow,
    SvelteDocument,
    SvelteFragment,
    SvelteBoundary,
    SvelteOptionsRaw,

    // Slots
    SlotElement,

    // Blocks
    IfBlock,
    EachBlock,
    AwaitBlock,
    KeyBlock,
    SnippetBlock,

    // Other
    Expression,
}

impl<'s, 'a, 'b> AnalysisVisitor<'s, 'a, 'b> {
    pub fn new(analysis: &'b mut ComponentAnalysis<'s>, scope: ScopeId, ast_type: AstType) -> Self {
        Self {
            state: AnalysisState::new(analysis, scope, ast_type),
            path: Vec::new(),
        }
    }

    /// Enters a new scope for analysis.
    fn enter_scope(&mut self, scope: ScopeId) {
        self.state.scope = scope;
    }

    /// Visits a JS expression.
    fn visit_js_expression(&mut self, expr: &Expression<'a>) {
        let mut js_visitor = JsAnalysisVisitor { parent: self };
        js_visitor.visit_expression(expr);
    }
}

impl<'s, 'a, 'b> SvelteVisitor<'a> for AnalysisVisitor<'s, 'a, 'b> {
    fn visit_root(&mut self, node: &Root<'a>) {
        self.path.push(NodeKind::Root);

        // Visit module script
        if let Some(ref module) = node.module {
            self.state.ast_type = AstType::Module;
            self.visit_script(module);
        }

        // Visit instance script
        if let Some(ref instance) = node.instance {
            self.state.ast_type = AstType::Instance;
            self.visit_script(instance);
        }

        // Visit template
        self.state.ast_type = AstType::Template;
        self.visit_fragment(&node.fragment);

        self.path.pop();
    }

    fn visit_script(&mut self, node: &Script<'a>) {
        self.path.push(NodeKind::Script);

        // Visit JS program
        let mut js_visitor = JsAnalysisVisitor { parent: self };
        js_visitor.visit_program(&node.content);

        self.path.pop();
    }

    fn visit_fragment(&mut self, node: &Fragment<'a>) {
        self.path.push(NodeKind::Fragment);
        visitor::walk_fragment(self, node);
        self.path.pop();
    }

    fn visit_expression_tag(&mut self, node: &ExpressionTag<'a>) {
        self.path.push(NodeKind::Expression);

        // Call the ExpressionTag visitor for validation
        visit_expression_tag(node, &mut self.state, &self.path);

        // Mark that we're in a template expression for await validation
        let was_in_template_expression = self.state.in_template_expression;
        self.state.in_template_expression = true;

        // Set up expression metadata tracking
        let expr_span = node.expression.span().into();
        let prev_expression = self.state.current_expression;
        self.state.current_expression = Some(expr_span);

        // Initialize expression metadata
        self.state.analysis.get_or_create_expression_meta(expr_span);

        self.visit_js_expression(&node.expression);

        // Restore previous state
        self.state.current_expression = prev_expression;
        self.state.in_template_expression = was_in_template_expression;
        self.path.pop();
    }

    fn visit_html_tag(&mut self, node: &HtmlTag<'a>) {
        // Call the HtmlTag visitor for validation
        visit_html_tag(node, &mut self.state, &self.path);

        // Visit the expression
        self.visit_js_expression(&node.expression);
    }

    fn visit_debug_tag(&mut self, node: &DebugTag<'a>) {
        // Call the DebugTag visitor for validation
        visit_debug_tag(node, &mut self.state, &self.path);

        // Visit each identifier in the debug tag
        for ident in &node.identifiers {
            self.visit_js_expression(ident);
        }
    }

    fn visit_render_tag(&mut self, node: &RenderTag<'a>) {
        // Call the RenderTag visitor for validation
        visit_render_tag(node, &mut self.state, &self.path);

        // Visit the expression (which contains the call with arguments)
        self.visit_js_expression(&node.expression);
    }

    fn visit_const_tag(&mut self, node: &ConstTag<'a>) {
        // Call the ConstTag visitor for validation
        visit_const_tag(node, &mut self.state, &self.path);

        // Visit the declaration
        let mut js_visitor = JsAnalysisVisitor { parent: self };
        js_visitor.visit_variable_declaration(&node.declaration);
    }

    fn visit_if_block(&mut self, node: &IfBlock<'a>) {
        self.path.push(NodeKind::IfBlock);

        // Call the IfBlock visitor for validation
        visit_if_block(node, &mut self.state, &self.path);

        // Visit the test expression
        self.visit_js_expression(&node.test);

        // Visit consequent and alternate
        self.visit_fragment(&node.consequent);
        if let Some(ref alternate) = node.alternate {
            self.visit_fragment(alternate);
        }

        self.path.pop();
    }

    fn visit_each_block(&mut self, node: &EachBlock<'a>) {
        self.path.push(NodeKind::EachBlock);

        // Call the EachBlock visitor for validation
        visit_each_block(node, &mut self.state, &self.path);

        // Visit the expression being iterated
        self.visit_js_expression(&node.expression);

        // Visit key expression if present
        if let Some(ref key) = node.key {
            self.visit_js_expression(key);
        }

        // Visit body and fallback
        self.visit_fragment(&node.body);
        if let Some(ref fallback) = node.fallback {
            self.visit_fragment(fallback);
        }

        self.path.pop();
    }

    fn visit_await_block(&mut self, node: &AwaitBlock<'a>) {
        self.path.push(NodeKind::AwaitBlock);

        // Call the AwaitBlock visitor for validation
        visit_await_block(node, &mut self.state, &self.path);

        // Visit the expression being awaited
        self.visit_js_expression(&node.expression);

        // Visit pending, then, and catch fragments
        if let Some(ref pending) = node.pending {
            self.visit_fragment(pending);
        }
        if let Some(ref then) = node.then {
            self.visit_fragment(then);
        }
        if let Some(ref catch) = node.catch {
            self.visit_fragment(catch);
        }

        self.path.pop();
    }

    fn visit_key_block(&mut self, node: &KeyBlock<'a>) {
        self.path.push(NodeKind::KeyBlock);

        // Call the KeyBlock visitor for validation
        visit_key_block(node, &mut self.state, &self.path);

        // Visit the key expression
        self.visit_js_expression(&node.expression);

        // Visit the fragment
        self.visit_fragment(&node.fragment);

        self.path.pop();
    }

    fn visit_snippet_block(&mut self, node: &lux_ast::blocks::SnippetBlock<'a>) {
        self.path.push(NodeKind::SnippetBlock);

        // Call the SnippetBlock visitor for validation
        visit_snippet_block(node, &mut self.state, &self.path);

        visitor::walk_snippet_block(self, node);
        self.path.pop();
    }

    fn visit_regular_element(&mut self, node: &RegularElement<'a>) {
        self.path.push(NodeKind::RegularElement(node.name));

        // Call the RegularElement visitor for validation and metadata
        visit_regular_element(node, &mut self.state, &self.path);

        visitor::walk_regular_element(self, node);
        self.path.pop();
    }

    fn visit_component(&mut self, node: &Component<'a>) {
        self.path.push(NodeKind::Component(node.name));

        // Call the Component visitor for validation and metadata
        visit_component(node, &mut self.state, &self.path);

        visitor::walk_component(self, node);
        self.path.pop();
    }

    fn visit_svelte_element(&mut self, node: &SvelteElement<'a>) {
        self.path.push(NodeKind::SvelteElement);

        // Call the SvelteElement visitor for validation and metadata
        visit_svelte_element(node, &mut self.state, &self.path);

        // Validate element attributes
        validate_element(&node.attributes, &mut self.state, &self.path);

        visitor::walk_svelte_element(self, node);
        self.path.pop();
    }

    fn visit_svelte_component(&mut self, node: &SvelteComponent<'a>) {
        self.path.push(NodeKind::SvelteComponent);

        // Call the SvelteComponent visitor for validation
        visit_svelte_component(node, &mut self.state, &self.path);

        visitor::walk_svelte_component(self, node);
        self.path.pop();
    }

    fn visit_svelte_self(&mut self, node: &SvelteSelf<'a>) {
        self.path.push(NodeKind::SvelteSelf);

        // Call the SvelteSelf visitor for validation
        visit_svelte_self(node, &mut self.state, &self.path);

        visitor::walk_svelte_self(self, node);
        self.path.pop();
    }

    fn visit_slot_element(&mut self, node: &SlotElement<'a>) {
        self.path.push(NodeKind::SlotElement);

        // Call the SlotElement visitor for validation and metadata
        visit_slot_element(node, &mut self.state, &self.path);

        visitor::walk_slot_element(self, node);
        self.path.pop();
    }

    fn visit_svelte_head(&mut self, node: &SvelteHead<'a>) {
        self.path.push(NodeKind::SvelteHead);

        // Call the SvelteHead visitor for validation
        visit_svelte_head(node, &mut self.state, &self.path);

        visitor::walk_svelte_head(self, node);
        self.path.pop();
    }

    fn visit_svelte_body(&mut self, node: &SvelteBody<'a>) {
        self.path.push(NodeKind::SvelteBody);

        // Call the SvelteBody visitor for validation
        visit_svelte_body(node, &mut self.state, &self.path);

        visitor::walk_svelte_body(self, node);
        self.path.pop();
    }

    fn visit_svelte_window(&mut self, node: &SvelteWindow<'a>) {
        self.path.push(NodeKind::SvelteWindow);

        // Call the SvelteWindow visitor for validation
        visit_svelte_window(node, &mut self.state, &self.path);

        visitor::walk_svelte_window(self, node);
        self.path.pop();
    }

    fn visit_svelte_document(&mut self, node: &SvelteDocument<'a>) {
        self.path.push(NodeKind::SvelteDocument);

        // Call the SvelteDocument visitor for validation
        visit_svelte_document(node, &mut self.state, &self.path);

        visitor::walk_svelte_document(self, node);
        self.path.pop();
    }

    fn visit_svelte_fragment(&mut self, node: &SvelteFragment<'a>) {
        self.path.push(NodeKind::SvelteFragment);

        // Call the SvelteFragment visitor for validation
        visit_svelte_fragment(node, &mut self.state, &self.path);

        visitor::walk_svelte_fragment(self, node);
        self.path.pop();
    }

    fn visit_svelte_boundary(&mut self, node: &SvelteBoundary<'a>) {
        self.path.push(NodeKind::SvelteBoundary);

        // Call the SvelteBoundary visitor for validation
        visit_svelte_boundary(node, &mut self.state, &self.path);

        visitor::walk_svelte_boundary(self, node);
        self.path.pop();
    }

    fn visit_title_element(&mut self, node: &TitleElement<'a>) {
        self.path.push(NodeKind::TitleElement);

        // Call the TitleElement visitor for validation
        visit_title_element(node, &mut self.state, &self.path);

        visitor::walk_title_element(self, node);
        self.path.pop();
    }

    fn visit_bind_directive(&mut self, node: &BindDirective<'a>) {
        visit_bind_directive(node, &mut self.state, &self.path);
        visitor::walk_bind_directive(self, node);
    }

    fn visit_on_directive(&mut self, node: &OnDirective<'a>) {
        visit_on_directive(node, &mut self.state, &self.path);

        if let Some(ref expr) = node.expression {
            self.visit_js_expression(expr);
        }
    }

    fn visit_class_directive(&mut self, node: &ClassDirective<'a>) {
        visit_class_directive(node, &mut self.state, &self.path);
        visitor::walk_class_directive(self, node);
    }

    fn visit_style_directive(&mut self, node: &StyleDirective<'a>) {
        visit_style_directive(node, &mut self.state, &self.path);
        visitor::walk_style_directive(self, node);
    }

    fn visit_transition_directive(&mut self, node: &TransitionDirective<'a>) {
        visit_transition_directive(node, &mut self.state, &self.path);
        visitor::walk_transition_directive(self, node);
    }

    fn visit_animate_directive(&mut self, node: &AnimateDirective<'a>) {
        visit_animate_directive(node, &mut self.state, &self.path);
        visitor::walk_animate_directive(self, node);
    }

    fn visit_use_directive(&mut self, node: &UseDirective<'a>) {
        visit_use_directive(node, &mut self.state, &self.path);
        visitor::walk_use_directive(self, node);
    }

    fn visit_let_directive(&mut self, node: &LetDirective<'a>) {
        visit_let_directive(node, &mut self.state, &self.path);
        visitor::walk_let_directive(self, node);
    }

    fn visit_spread_attribute(&mut self, node: &SpreadAttribute<'a>) {
        visit_spread_attribute(node, &mut self.state, &self.path);
        visitor::walk_spread_attribute(self, node);
    }

    fn visit_text(&mut self, node: &lux_ast::text::Text<'a>) {
        visit_text(node, &mut self.state, &self.path);
    }
}

/// JS visitor that calls back into the analysis visitor.
struct JsAnalysisVisitor<'s, 'a, 'b, 'c> {
    parent: &'c mut AnalysisVisitor<'s, 'a, 'b>,
}

impl<'a> Visit<'a> for JsAnalysisVisitor<'_, 'a, '_, '_> {
    fn visit_identifier_reference(&mut self, node: &IdentifierReference<'a>) {
        visit_identifier(node, &mut self.parent.state);

        let state = &mut self.parent.state;

        // Track reactive statement dependencies (legacy mode)
        if let Some(reactive_span) = state.reactive_statement {
            if let Some(binding_id) = state.analysis.scope_tree.get(state.scope, &node.name) {
                // Only track if not inside a nested function
                if state.function_depth <= 1 {
                    if let Some(reactive_stmt) = state.analysis.reactive_statements.get_mut(&reactive_span) {
                        reactive_stmt.dependencies.push(binding_id);
                    }
                }
            }
        }

        // Track expression metadata: add binding as dependency and check for state
        if let Some(expr_span) = state.current_expression {
            if let Some(binding_id) = state.analysis.scope_tree.get(state.scope, &node.name) {
                // Extract binding kind before mutating (to avoid borrow conflict)
                let binding_kind = state.analysis.scope_tree.get_binding(binding_id).kind;

                // Add binding to dependencies and references
                state.analysis.add_expression_dependency(expr_span, binding_id);

                // Mark has_state for state-like bindings
                let has_state_kind = matches!(
                    binding_kind,
                    BindingKind::State
                        | BindingKind::RawState
                        | BindingKind::Derived
                        | BindingKind::Prop
                        | BindingKind::BindableProp
                        | BindingKind::RestProp
                );
                if has_state_kind {
                    state.analysis.mark_expression_has_state(expr_span);
                }
            }
        }
    }

    fn visit_member_expression(&mut self, node: &MemberExpression<'a>) {
        let state = &mut self.parent.state;

        // Track expression metadata: has_member_expression
        if let Some(expr_span) = state.current_expression {
            state.analysis.mark_expression_has_member(expr_span);
            // Member expressions may access state indirectly
            state.analysis.mark_expression_has_state(expr_span);
        }

        // Check for rest_prop accessing $$ properties
        if let MemberExpression::StaticMemberExpression(static_member) = node {
            if let Expression::Identifier(object) = &static_member.object {
                let property_name = static_member.property.name.as_str();

                // Check if accessing a rest_prop with $$ prefix
                if property_name.starts_with("$$") {
                    if let Some(binding_id) = state.analysis.scope_tree.get(state.scope, &object.name) {
                        let binding = state.analysis.scope_tree.get_binding(binding_id);
                        if binding.kind == crate::scope::BindingKind::RestProp {
                            state.analysis.error(
                                super::errors::props_illegal_name(static_member.property.span.into()),
                            );
                        }
                    }
                }
            }
        }

        // Mark that we need context for certain expressions
        state.analysis.needs_context = true;

        walk::walk_member_expression(self, node);
    }

    fn visit_call_expression(&mut self, node: &CallExpression<'a>) {
        visit_call_expression(node, &mut self.parent.state);

        // Track expression metadata: has_call and has_state (calls can return state)
        let state = &mut self.parent.state;
        if let Some(expr_span) = state.current_expression {
            state.analysis.mark_expression_has_call(expr_span);
            state.analysis.mark_expression_has_state(expr_span);
        }

        walk::walk_call_expression(self, node);
    }

    fn visit_new_expression(&mut self, node: &NewExpression<'a>) {
        let state = &mut self.parent.state;

        // Warn about inline class creation (performance issue)
        if matches!(&node.callee, Expression::ClassExpression(_)) {
            if state.function_depth > 0 {
                state.analysis.warning(
                    super::warnings::perf_avoid_inline_class(node.span.into()),
                );
            }
        }

        // Check for legacy component instantiation pattern:
        // new Component({ target: ... })
        if let Expression::Identifier(callee) = &node.callee {
            if node.arguments.len() == 1 {
                if let Some(oxc_ast::ast::Argument::ObjectExpression(obj)) = node.arguments.first() {
                    let has_target_prop = obj.properties.iter().any(|prop| {
                        if let oxc_ast::ast::ObjectPropertyKind::ObjectProperty(p) = prop {
                            if let oxc_ast::ast::PropertyKey::StaticIdentifier(key) = &p.key {
                                return key.name == "target";
                            }
                        }
                        false
                    });

                    if has_target_prop {
                        // Check if the callee is an imported Svelte component
                        if let Some(binding_id) = state.analysis.scope_tree.get(state.scope, &callee.name) {
                            let binding = state.analysis.scope_tree.get_binding(binding_id);
                            if binding.kind == BindingKind::Normal
                                && binding.declaration_kind == crate::scope::DeclarationKind::Import
                            {
                                // TODO: Check if import source ends with .svelte
                                // For now, just emit the warning for any imported component-like pattern
                                state.analysis.warning(
                                    super::warnings::legacy_component_creation(node.span.into()),
                                );
                            }
                        }
                    }
                }
            }
        }

        state.analysis.needs_context = true;

        walk::walk_new_expression(self, node);
    }

    fn visit_update_expression(&mut self, node: &UpdateExpression<'a>) {
        // Track expression metadata: has_assignment
        let state = &mut self.parent.state;
        if let Some(expr_span) = state.current_expression {
            state.analysis.mark_expression_has_assignment(expr_span);
        }

        // Validate the assignment target
        self.validate_simple_assignment_target(&node.argument, node.span, false);

        walk::walk_update_expression(self, node);
    }

    fn visit_assignment_expression(&mut self, node: &AssignmentExpression<'a>) {
        let state = &mut self.parent.state;

        // Track expression metadata: has_assignment
        if let Some(expr_span) = state.current_expression {
            state.analysis.mark_expression_has_assignment(expr_span);
        }

        // Track reactive statement assignments (legacy mode)
        if let Some(reactive_span) = state.reactive_statement {
            // Extract identifiers from the left side of the assignment
            if let oxc_ast::ast::AssignmentTarget::AssignmentTargetIdentifier(id) = &node.left {
                if let Some(binding_id) = state.analysis.scope_tree.get(state.scope, &id.name) {
                    if let Some(reactive_stmt) = state.analysis.reactive_statements.get_mut(&reactive_span) {
                        reactive_stmt.assignments.insert(binding_id);
                    }
                }
            }
        }

        // Validate the assignment target
        self.validate_assignment_pattern(&node.left, node.span, false);

        walk::walk_assignment_expression(self, node);
    }

    fn visit_await_expression(&mut self, node: &AwaitExpression<'a>) {
        let state = &mut self.parent.state;

        // Track expression metadata: has_await
        if let Some(expr_span) = state.current_expression {
            state.analysis.mark_expression_has_await(expr_span);
        }

        // Check for top-level await in instance script
        // function_depth == 0 means we're at the script body level, not inside any function
        let is_tla = matches!(state.ast_type, AstType::Instance) && state.function_depth == 0;

        // Check if we're in a template expression context
        let in_template_expression = state.in_template_expression;

        // Determine if we need to check for experimental async
        let suspend = is_tla || in_template_expression;

        if suspend {
            // TODO: Check experimental.async option
            // For now, we assume experimental.async is not enabled
            let experimental_async = false;

            if !experimental_async {
                state.analysis.error(
                    super::errors::experimental_async(node.span.into()),
                );
            }

            if !state.analysis.runes {
                state.analysis.error(
                    super::errors::legacy_await_invalid(node.span.into()),
                );
            }
        }

        walk::walk_await_expression(self, node);
    }

    fn visit_variable_declarator(&mut self, node: &VariableDeclarator<'a>) {
        let state = &mut self.parent.state;

        if state.analysis.runes {
            // Check for rune usage
            if let Some(ref init) = node.init {
                let rune = get_rune(init, &state.analysis.scope_tree);

                // Validate $props pattern
                if rune == Some("$props") {
                    match &node.id {
                        BindingPattern::ObjectPattern(_) | BindingPattern::BindingIdentifier(_) => {
                            // Valid patterns
                        }
                        _ => {
                            state.analysis.error(
                                super::errors::props_invalid_identifier(node.id.span().into()),
                            );
                        }
                    }

                    state.analysis.needs_props = true;

                    // Check for $$ prefixed property names in object pattern
                    if let BindingPattern::ObjectPattern(obj) = &node.id {
                        for prop in &obj.properties {
                            if let Some(key_name) = get_property_key_name(&prop.key) {
                                if key_name.starts_with("$$") {
                                    state.analysis.error(
                                        super::errors::props_illegal_name(prop.key.span().into()),
                                    );
                                }
                            }

                            // Check for computed non-literal keys
                            if prop.computed {
                                if !matches!(
                                    &prop.key,
                                    oxc_ast::ast::PropertyKey::StringLiteral(_)
                                        | oxc_ast::ast::PropertyKey::NumericLiteral(_)
                                ) {
                                    state.analysis.error(
                                        super::errors::props_invalid_pattern(prop.key.span().into()),
                                    );
                                }
                            }

                            // Validate that value is an identifier (possibly with default)
                            let value_pattern = match &prop.value {
                                BindingPattern::AssignmentPattern(assign) => &assign.left,
                                other => other,
                            };
                            if !matches!(value_pattern, BindingPattern::BindingIdentifier(_)) {
                                state.analysis.error(
                                    super::errors::props_invalid_pattern(prop.value.span().into()),
                                );
                            }
                        }
                    }

                    // Update binding kinds for $props
                    for id in extract_identifiers_from_pattern(&node.id) {
                        if let Some(binding_id) = state.analysis.scope_tree
                            .get(state.scope, &id.name)
                        {
                            let binding = state.analysis.scope_tree.get_binding_mut(binding_id);
                            // Determine if it's a rest prop
                            let is_rest = is_rest_element(&node.id, &id.name);
                            binding.kind = if is_rest {
                                BindingKind::RestProp
                            } else {
                                BindingKind::Prop
                            };
                        }
                    }
                } else if matches!(rune, Some("$state") | Some("$state.raw") | Some("$derived") | Some("$derived.by")) {
                    // Update binding kinds for state/derived
                    for id in extract_identifiers_from_pattern(&node.id) {
                        if let Some(binding_id) = state.analysis.scope_tree
                            .get(state.scope, &id.name)
                        {
                            let binding = state.analysis.scope_tree.get_binding_mut(binding_id);
                            binding.kind = match rune {
                                Some("$state") => BindingKind::State,
                                Some("$state.raw") => BindingKind::RawState,
                                Some("$derived") | Some("$derived.by") => BindingKind::Derived,
                                _ => binding.kind,
                            };
                        }
                    }
                }
            }
        } else {
            // Non-runes mode: check for invalid rune usage
            if let Some(Expression::CallExpression(call)) = &node.init {
                if let Expression::Identifier(callee) = &call.callee {
                    let name = callee.name.as_str();
                    if matches!(name, "$state" | "$derived" | "$props") {
                        // Check if this is actually a store subscription
                        let is_store_sub = state.analysis.scope_tree
                            .get(state.scope, name)
                            .map(|binding_id| {
                                let binding = state.analysis.scope_tree.get_binding(binding_id);
                                binding.kind == BindingKind::StoreSub
                            })
                            .unwrap_or(false);

                        if !is_store_sub {
                            state.analysis.error(
                                super::errors::rune_invalid_usage(call.span.into(), name),
                            );
                        }
                    }
                }
            }
        }

        // Validate identifier names ($ prefix validation)
        for id in extract_identifiers_from_pattern(&node.id) {
            if let Some(binding_id) = state.analysis.scope_tree.get(state.scope, &id.name) {
                let binding = state.analysis.scope_tree.get_binding(binding_id);
                let function_depth = if state.analysis.runes {
                    None // In runes mode, always validate
                } else {
                    Some(state.function_depth as usize)
                };

                if let Some(error) = validate_identifier_name(binding, function_depth) {
                    match error {
                        IdentifierNameError::DollarBinding => {
                            state.analysis.error(
                                super::errors::dollar_binding_invalid(id.span.into()),
                            );
                        }
                        IdentifierNameError::DollarPrefix => {
                            state.analysis.error(
                                super::errors::dollar_prefix_invalid(id.span.into()),
                            );
                        }
                    }
                }
            }
        }

        walk::walk_variable_declarator(self, node);
    }

    fn visit_expression_statement(&mut self, node: &ExpressionStatement<'a>) {
        // Check for legacy component creation pattern in expression statements
        // new Component({ target: ... })
        // This is handled in visit_new_expression, so we just walk here

        walk::walk_expression_statement(self, node);
    }

    fn visit_labeled_statement(&mut self, node: &LabeledStatement<'a>) {
        // Handle $: reactive statements
        if node.label.name == "$" {
            let is_in_instance = matches!(self.parent.state.ast_type, AstType::Instance);
            // Check if we're at the top level (function_depth == 0)
            let is_top_level = self.parent.state.function_depth == 0;

            let is_reactive_statement = is_in_instance && is_top_level;

            if is_reactive_statement {
                if self.parent.state.analysis.runes {
                    // In runes mode, $: reactive statements are not allowed
                    self.parent.state.analysis.error(
                        super::errors::legacy_reactive_statement_invalid(node.span.into()),
                    );
                } else {
                    // In legacy mode, track this reactive statement
                    let stmt_span: Span = node.span.into();
                    let prev_reactive = self.parent.state.reactive_statement;
                    self.parent.state.reactive_statement = Some(stmt_span);

                    // Initialize the reactive statement entry
                    self.parent.state.analysis.reactive_statements.insert(
                        stmt_span,
                        super::ReactiveStatement::default(),
                    );

                    // Walk the body with reactive_statement context set
                    // Increase function_depth to prevent nested reactive warnings
                    self.parent.state.function_depth += 1;
                    walk::walk_labeled_statement(self, node);
                    self.parent.state.function_depth -= 1;

                    // Restore previous state
                    self.parent.state.reactive_statement = prev_reactive;
                    return;
                }
            } else if !self.parent.state.analysis.runes {
                // Warning for $: in wrong position (only in legacy mode)
                self.parent.state.analysis.warning(
                    super::warnings::reactive_declaration_invalid_placement(node.span.into()),
                );
            }
        }

        walk::walk_labeled_statement(self, node);
    }

    fn visit_function(
        &mut self,
        func: &oxc_ast::ast::Function<'a>,
        _flags: ScopeFlags,
    ) {
        let state = &mut self.parent.state;

        // Validate identifier name for function declarations in runes mode
        if state.analysis.runes {
            if let Some(ref id) = func.id {
                if let Some(binding_id) = state.analysis.scope_tree.get(state.scope, &id.name) {
                    let binding = state.analysis.scope_tree.get_binding(binding_id);
                    if let Some(error) = validate_identifier_name(binding, None) {
                        match error {
                            IdentifierNameError::DollarBinding => {
                                state.analysis.error(
                                    super::errors::dollar_binding_invalid(id.span.into()),
                                );
                            }
                            IdentifierNameError::DollarPrefix => {
                                state.analysis.error(
                                    super::errors::dollar_prefix_invalid(id.span.into()),
                                );
                            }
                        }
                    }
                }
            }
        }

        self.parent.state.function_depth += 1;
        walk::walk_function(self, func, ScopeFlags::empty());
        self.parent.state.function_depth -= 1;
    }

    fn visit_arrow_function_expression(
        &mut self,
        func: &oxc_ast::ast::ArrowFunctionExpression<'a>,
    ) {
        self.parent.state.function_depth += 1;
        walk::walk_arrow_function_expression(self, func);
        self.parent.state.function_depth -= 1;
    }

    fn visit_import_declaration(&mut self, node: &ImportDeclaration<'a>) {
        let state = &mut self.parent.state;

        if state.analysis.runes {
            let source = node.source.value.as_str();

            // Forbid imports from svelte/internal
            if source.starts_with("svelte/internal") {
                state.analysis.error(
                    super::errors::import_svelte_internal_forbidden(node.span.into()),
                );
            }

            // Check for invalid imports from 'svelte' in runes mode
            if source == "svelte" {
                if let Some(specifiers) = &node.specifiers {
                    for specifier in specifiers.iter() {
                        if let ImportDeclarationSpecifier::ImportSpecifier(spec) = specifier {
                            let imported_name = spec.imported.name().as_str();
                            if imported_name == "beforeUpdate" || imported_name == "afterUpdate" {
                                state.analysis.error(
                                    super::errors::runes_mode_invalid_import(
                                        spec.span.into(),
                                        imported_name,
                                    ),
                                );
                            }
                        }
                    }
                }
            }
        }

        walk::walk_import_declaration(self, node);
    }

    fn visit_export_named_declaration(&mut self, node: &ExportNamedDeclaration<'a>) {
        // Visit children first so bindings are correctly initialized
        walk::walk_export_named_declaration(self, node);

        // Check for default export
        if matches!(self.parent.state.ast_type, AstType::Module | AstType::Instance) {
            let has_default_export = node.specifiers.iter().any(|specifier| {
                match &specifier.exported {
                    ModuleExportName::IdentifierName(name) => name.name == "default",
                    ModuleExportName::IdentifierReference(id) => id.name == "default",
                    ModuleExportName::StringLiteral(lit) => lit.value == "default",
                }
            });
            if has_default_export {
                self.parent.state.analysis.error(
                    super::errors::module_illegal_default_export(node.span.into()),
                );
            }
        }

        // Handle variable declaration exports
        if let Some(Declaration::VariableDeclaration(ref decl)) = node.declaration {
            // In runes mode, forbid `export let`
            if self.parent.state.analysis.runes
                && matches!(self.parent.state.ast_type, AstType::Instance)
                && matches!(decl.kind, VariableDeclarationKind::Let)
            {
                self.parent.state.analysis.error(
                    super::errors::legacy_export_invalid(node.span.into()),
                );
            }

            // Check for derived/state exports
            for declarator in &decl.declarations {
                for id in extract_identifiers_from_pattern(&declarator.id) {
                    let name = id.name.as_str();

                    // Get binding and check its kind (copy values to avoid borrow issues)
                    let binding_info = self.parent.state.analysis.scope_tree
                        .get(self.parent.state.scope, name)
                        .map(|binding_id| {
                            let binding = self.parent.state.analysis.scope_tree.get_binding(binding_id);
                            (binding.kind, binding.reassigned)
                        });

                    if let Some((kind, reassigned)) = binding_info {
                        if kind == BindingKind::Derived {
                            self.parent.state.analysis.error(
                                super::errors::derived_invalid_export(node.span.into()),
                            );
                        }

                        if matches!(kind, BindingKind::State | BindingKind::RawState) && reassigned {
                            self.parent.state.analysis.error(
                                super::errors::state_invalid_export(node.span.into()),
                            );
                        }
                    }
                }
            }

            // In runes mode, track exports for const declarations
            if self.parent.state.analysis.runes
                && matches!(self.parent.state.ast_type, AstType::Instance)
                && matches!(decl.kind, VariableDeclarationKind::Const)
            {
                for declarator in &decl.declarations {
                    for id in extract_identifiers_from_pattern(&declarator.id) {
                        self.parent.state.analysis.exports.push(super::Export {
                            name: id.name.to_string(),
                            alias: None,
                        });
                    }
                }
            }
        }

        // Handle function/class declaration exports in runes mode
        if self.parent.state.analysis.runes
            && matches!(self.parent.state.ast_type, AstType::Instance)
        {
            if let Some(ref declaration) = node.declaration {
                match declaration {
                    Declaration::FunctionDeclaration(func) => {
                        if let Some(ref id) = func.id {
                            self.parent.state.analysis.exports.push(super::Export {
                                name: id.name.to_string(),
                                alias: None,
                            });
                        }
                    }
                    Declaration::ClassDeclaration(class) => {
                        if let Some(ref id) = class.id {
                            self.parent.state.analysis.exports.push(super::Export {
                                name: id.name.to_string(),
                                alias: None,
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

impl<'s, 'a, 'b, 'c> JsAnalysisVisitor<'s, 'a, 'b, 'c> {
    /// Validates a simple assignment target (for UpdateExpression).
    fn validate_simple_assignment_target(&mut self, target: &oxc_ast::ast::SimpleAssignmentTarget<'a>, span: oxc_span::Span, is_binding: bool) {
        use oxc_ast::ast::SimpleAssignmentTarget;

        match target {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(id) => {
                self.validate_identifier_assignment(&id.name, span, is_binding);
            }
            // Member expressions don't need const assignment validation
            _ => {}
        }
    }

    /// Validates an assignment pattern (for AssignmentExpression).
    fn validate_assignment_pattern(&mut self, target: &oxc_ast::ast::AssignmentTarget<'a>, span: oxc_span::Span, is_binding: bool) {
        use oxc_ast::ast::AssignmentTarget;

        match target {
            AssignmentTarget::AssignmentTargetIdentifier(id) => {
                self.validate_identifier_assignment(&id.name, span, is_binding);
            }
            AssignmentTarget::ArrayAssignmentTarget(arr) => {
                for elem in arr.elements.iter().flatten() {
                    self.validate_assignment_target_maybe_default(elem, span, is_binding);
                }
                if let Some(rest) = &arr.rest {
                    self.validate_assignment_target_rest(rest, span, is_binding);
                }
            }
            AssignmentTarget::ObjectAssignmentTarget(obj) => {
                for prop in &obj.properties {
                    match prop {
                        oxc_ast::ast::AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(id) => {
                            self.validate_identifier_assignment(&id.binding.name, span, is_binding);
                        }
                        oxc_ast::ast::AssignmentTargetProperty::AssignmentTargetPropertyProperty(prop) => {
                            self.validate_assignment_target_maybe_default(&prop.binding, span, is_binding);
                        }
                    }
                }
                if let Some(rest) = &obj.rest {
                    self.validate_assignment_target_rest(rest, span, is_binding);
                }
            }
            _ => {
                // MemberExpression targets don't need const assignment validation
            }
        }
    }

    fn validate_assignment_target_maybe_default(&mut self, target: &oxc_ast::ast::AssignmentTargetMaybeDefault<'a>, span: oxc_span::Span, is_binding: bool) {
        use oxc_ast::ast::AssignmentTargetMaybeDefault;
        match target {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(with_default) => {
                self.validate_assignment_pattern(&with_default.binding, span, is_binding);
            }
            AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(id) => {
                self.validate_identifier_assignment(&id.name, span, is_binding);
            }
            AssignmentTargetMaybeDefault::StaticMemberExpression(_) |
            AssignmentTargetMaybeDefault::ComputedMemberExpression(_) |
            AssignmentTargetMaybeDefault::PrivateFieldExpression(_) => {
                // Member expressions don't need const assignment validation
            }
            AssignmentTargetMaybeDefault::ArrayAssignmentTarget(arr) => {
                for elem in arr.elements.iter().flatten() {
                    self.validate_assignment_target_maybe_default(elem, span, is_binding);
                }
            }
            AssignmentTargetMaybeDefault::ObjectAssignmentTarget(obj) => {
                for prop in &obj.properties {
                    match prop {
                        oxc_ast::ast::AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(id) => {
                            self.validate_identifier_assignment(&id.binding.name, span, is_binding);
                        }
                        oxc_ast::ast::AssignmentTargetProperty::AssignmentTargetPropertyProperty(prop) => {
                            self.validate_assignment_target_maybe_default(&prop.binding, span, is_binding);
                        }
                    }
                }
            }
            AssignmentTargetMaybeDefault::TSAsExpression(_) |
            AssignmentTargetMaybeDefault::TSSatisfiesExpression(_) |
            AssignmentTargetMaybeDefault::TSNonNullExpression(_) |
            AssignmentTargetMaybeDefault::TSTypeAssertion(_) => {
                // TypeScript expressions, skip
            }
        }
    }

    fn validate_assignment_target_rest(&mut self, target: &oxc_ast::ast::AssignmentTargetRest<'a>, span: oxc_span::Span, is_binding: bool) {
        self.validate_assignment_pattern(&target.target, span, is_binding);
    }

    /// Validates that an identifier can be assigned to.
    fn validate_identifier_assignment(&mut self, name: &str, span: oxc_span::Span, is_binding: bool) {
        let state = &mut self.parent.state;

        // Look up binding info (copy to avoid borrow issues)
        let binding_info = state.analysis.scope_tree
            .get(state.scope, name)
            .map(|binding_id| {
                let binding = state.analysis.scope_tree.get_binding(binding_id);
                (binding.kind, binding.declaration_kind)
            });

        if let Some((kind, declaration_kind)) = binding_info {
            use crate::scope::{DeclarationKind, BindingKind};

            // Check for const/import assignment
            if declaration_kind == DeclarationKind::Import
                || (declaration_kind == DeclarationKind::Const && kind != BindingKind::Each)
            {
                let thing = if declaration_kind == DeclarationKind::Import {
                    "import"
                } else {
                    "constant"
                };

                if is_binding {
                    state.analysis.error(super::errors::constant_binding(span.into(), thing));
                } else {
                    state.analysis.error(super::errors::constant_assignment(span.into(), thing));
                }
            }

            // In runes mode, check for each item assignment
            if state.analysis.runes && kind == BindingKind::Each {
                state.analysis.error(super::errors::each_item_invalid_assignment(span.into()));
            }

            // Check for snippet parameter assignment
            if kind == BindingKind::Snippet {
                state.analysis.error(super::errors::snippet_parameter_assignment(span.into()));
            }

            // Mark binding as reassigned
            if let Some(binding_id) = state.analysis.scope_tree.get(state.scope, name) {
                let binding = state.analysis.scope_tree.get_binding_mut(binding_id);
                binding.reassigned = true;
            }
        }
    }
}

/// Helper to get property key name
fn get_property_key_name<'a>(key: &'a oxc_ast::ast::PropertyKey<'a>) -> Option<&'a str> {
    match key {
        oxc_ast::ast::PropertyKey::StaticIdentifier(id) => Some(id.name.as_str()),
        oxc_ast::ast::PropertyKey::StringLiteral(lit) => Some(lit.value.as_str()),
        _ => None,
    }
}

/// Helper to check if an identifier is in a rest element position
fn is_rest_element(pattern: &BindingPattern<'_>, name: &str) -> bool {
    match pattern {
        BindingPattern::BindingIdentifier(_) => true, // Single identifier means rest_prop for $props
        BindingPattern::ObjectPattern(obj) => {
            if let Some(rest) = &obj.rest {
                if let BindingPattern::BindingIdentifier(id) = &rest.argument {
                    return id.name == name;
                }
            }
            false
        }
        BindingPattern::ArrayPattern(arr) => {
            if let Some(rest) = &arr.rest {
                if let BindingPattern::BindingIdentifier(id) = &rest.argument {
                    return id.name == name;
                }
            }
            false
        }
        BindingPattern::AssignmentPattern(assign) => is_rest_element(&assign.left, name),
    }
}

/// Runs the analysis visitor on the AST.
pub fn run_analysis<'s, 'a>(root: &'a Root<'a>, analysis: &mut ComponentAnalysis<'s>) {
    let scope = analysis.scope_tree.root_scope_id();
    let mut visitor = AnalysisVisitor::new(analysis, scope, AstType::Template);
    visitor.visit_root(root);
}
