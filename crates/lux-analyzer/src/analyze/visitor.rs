//! Analysis visitor that walks both Svelte and JS ASTs.
//!
//! This visitor performs the actual analysis work by traversing the AST
//! and collecting metadata, validating, and updating bindings.

use oxc_ast::ast::{
    AssignmentExpression, CallExpression, Expression, IdentifierReference, UpdateExpression,
};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;

use lux_ast::attributes::{BindDirective, OnDirective};
use lux_ast::blocks::{AwaitBlock, EachBlock, IfBlock, KeyBlock};
use lux_ast::elements::{
    Component, RegularElement, SlotElement, SvelteBody, SvelteBoundary, SvelteComponent,
    SvelteDocument, SvelteElement, SvelteFragment, SvelteHead, SvelteSelf, SvelteWindow,
    TitleElement,
};
use lux_ast::root::{Fragment, Root, Script};
use lux_ast::tags::{ExpressionTag, RenderTag};

use super::analysis::ComponentAnalysis;
use super::state::{AnalysisState, AstType};
use super::visitors::shared::{disallow_children, validate_element};
use super::visitors::{
    visit_await_block, visit_bind_directive, visit_call_expression, visit_each_block,
    visit_identifier, visit_if_block, visit_key_block, visit_on_directive, visit_render_tag,
    visit_snippet_block,
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
        self.visit_js_expression(&node.expression);
        self.path.pop();
    }

    fn visit_render_tag(&mut self, node: &RenderTag<'a>) {
        // Call the RenderTag visitor for validation
        visit_render_tag(node, &mut self.state, &self.path);

        // Visit the expression (which contains the call with arguments)
        self.visit_js_expression(&node.expression);
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

        // Validate element attributes
        validate_element(&node.attributes, &mut self.state, &self.path);

        visitor::walk_regular_element(self, node);
        self.path.pop();
    }

    fn visit_component(&mut self, node: &Component<'a>) {
        self.path.push(NodeKind::Component(node.name));
        visitor::walk_component(self, node);
        self.path.pop();
    }

    fn visit_svelte_element(&mut self, node: &SvelteElement<'a>) {
        self.path.push(NodeKind::SvelteElement);

        // Validate element attributes
        validate_element(&node.attributes, &mut self.state, &self.path);

        visitor::walk_svelte_element(self, node);
        self.path.pop();
    }

    fn visit_svelte_component(&mut self, node: &SvelteComponent<'a>) {
        self.path.push(NodeKind::SvelteComponent);
        visitor::walk_svelte_component(self, node);
        self.path.pop();
    }

    fn visit_svelte_self(&mut self, node: &SvelteSelf<'a>) {
        self.path.push(NodeKind::SvelteSelf);
        visitor::walk_svelte_self(self, node);
        self.path.pop();
    }

    fn visit_slot_element(&mut self, node: &SlotElement<'a>) {
        self.path.push(NodeKind::SlotElement);

        // Record slot name
        let slot_name = node
            .attributes
            .iter()
            .find_map(|attr| {
                if let lux_ast::node::AttributeNode::Attribute(a) = attr {
                    if a.name == "name" {
                        // TODO: extract actual name from value
                        return Some("default".to_string());
                    }
                }
                None
            })
            .unwrap_or_else(|| "default".to_string());

        self.state
            .analysis
            .slot_names
            .insert(slot_name, node.span.into());

        visitor::walk_slot_element(self, node);
        self.path.pop();
    }

    fn visit_svelte_head(&mut self, node: &SvelteHead<'a>) {
        self.path.push(NodeKind::SvelteHead);
        visitor::walk_svelte_head(self, node);
        self.path.pop();
    }

    fn visit_svelte_body(&mut self, node: &SvelteBody<'a>) {
        self.path.push(NodeKind::SvelteBody);

        // svelte:body cannot have children
        disallow_children(&node.fragment, "svelte:body", &mut self.state);

        visitor::walk_svelte_body(self, node);
        self.path.pop();
    }

    fn visit_svelte_window(&mut self, node: &SvelteWindow<'a>) {
        self.path.push(NodeKind::SvelteWindow);

        // svelte:window cannot have children
        disallow_children(&node.fragment, "svelte:window", &mut self.state);

        visitor::walk_svelte_window(self, node);
        self.path.pop();
    }

    fn visit_svelte_document(&mut self, node: &SvelteDocument<'a>) {
        self.path.push(NodeKind::SvelteDocument);

        // svelte:document cannot have children
        disallow_children(&node.fragment, "svelte:document", &mut self.state);

        visitor::walk_svelte_document(self, node);
        self.path.pop();
    }

    fn visit_svelte_fragment(&mut self, node: &SvelteFragment<'a>) {
        self.path.push(NodeKind::SvelteFragment);
        visitor::walk_svelte_fragment(self, node);
        self.path.pop();
    }

    fn visit_svelte_boundary(&mut self, node: &SvelteBoundary<'a>) {
        self.path.push(NodeKind::SvelteBoundary);
        visitor::walk_svelte_boundary(self, node);
        self.path.pop();
    }

    fn visit_title_element(&mut self, node: &TitleElement<'a>) {
        self.path.push(NodeKind::TitleElement);
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
}

/// JS visitor that calls back into the analysis visitor.
struct JsAnalysisVisitor<'s, 'a, 'b, 'c> {
    parent: &'c mut AnalysisVisitor<'s, 'a, 'b>,
}

impl<'a> Visit<'a> for JsAnalysisVisitor<'_, 'a, '_, '_> {
    fn visit_identifier_reference(&mut self, node: &IdentifierReference<'a>) {
        visit_identifier(node, &mut self.parent.state);
    }

    fn visit_call_expression(&mut self, node: &CallExpression<'a>) {
        visit_call_expression(node, &mut self.parent.state);
        walk::walk_call_expression(self, node);
    }

    fn visit_update_expression(&mut self, node: &UpdateExpression<'a>) {
        // TODO: track mutations
        walk::walk_update_expression(self, node);
    }

    fn visit_assignment_expression(&mut self, node: &AssignmentExpression<'a>) {
        // TODO: track mutations
        walk::walk_assignment_expression(self, node);
    }

    fn visit_function(
        &mut self,
        func: &oxc_ast::ast::Function<'a>,
        _flags: ScopeFlags,
    ) {
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
}

/// Runs the analysis visitor on the AST.
pub fn run_analysis<'s, 'a>(root: &'a Root<'a>, analysis: &mut ComponentAnalysis<'s>) {
    let scope = analysis.scope_tree.root_scope_id();
    let mut visitor = AnalysisVisitor::new(analysis, scope, AstType::Template);
    visitor.visit_root(root);
}
