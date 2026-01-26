//! AST visitor traits for Svelte analysis.
//!
//! This module provides visitor traits that can be implemented to traverse
//! both Svelte AST nodes and JavaScript/TypeScript nodes (via oxc).

use lux_ast::attributes::{
    AnimateDirective, Attribute, BindDirective, ClassDirective, LetDirective, OnDirective,
    SpreadAttribute, StyleDirective, TransitionDirective, UseDirective,
};
use lux_ast::blocks::{AwaitBlock, EachBlock, IfBlock, KeyBlock, SnippetBlock};
use lux_ast::elements::{
    Component, RegularElement, SlotElement, SvelteBody, SvelteBoundary, SvelteComponent,
    SvelteDocument, SvelteElement, SvelteFragment, SvelteHead, SvelteSelf, SvelteWindow,
    TitleElement,
};
use lux_ast::node::{AttributeNode, FragmentNode};
use lux_ast::root::{Fragment, Root, Script};
use lux_ast::tags::{AttachTag, ConstTag, DebugTag, ExpressionTag, HtmlTag, RenderTag};
use lux_ast::text::{Comment, Text};

/// A visitor for Svelte AST nodes.
///
/// Implement this trait to perform operations while traversing a Svelte AST.
/// Default implementations call the appropriate `walk_*` function to continue traversal.
/// Uses mutable references to allow modifying node metadata during analysis.
pub trait SvelteVisitor<'a> {
    // ========================================================================
    // Root & Script
    // ========================================================================

    fn visit_root(&mut self, node: &mut Root<'a>) {
        walk_root(self, node);
    }

    fn visit_script(&mut self, node: &mut Script<'a>) {
        walk_script(self, node);
    }

    fn visit_fragment(&mut self, node: &mut Fragment<'a>) {
        walk_fragment(self, node);
    }

    // ========================================================================
    // Fragment Nodes
    // ========================================================================

    fn visit_fragment_node(&mut self, node: &mut FragmentNode<'a>) {
        walk_fragment_node(self, node);
    }

    fn visit_text(&mut self, _node: &mut Text<'a>) {}

    fn visit_comment(&mut self, _node: &mut Comment<'a>) {}

    // Tags
    fn visit_expression_tag(&mut self, node: &mut ExpressionTag<'a>) {
        walk_expression_tag(self, node);
    }

    fn visit_html_tag(&mut self, node: &mut HtmlTag<'a>) {
        walk_html_tag(self, node);
    }

    fn visit_const_tag(&mut self, node: &mut ConstTag<'a>) {
        walk_const_tag(self, node);
    }

    fn visit_debug_tag(&mut self, node: &mut DebugTag<'a>) {
        walk_debug_tag(self, node);
    }

    fn visit_render_tag(&mut self, node: &mut RenderTag<'a>) {
        walk_render_tag(self, node);
    }

    fn visit_attach_tag(&mut self, node: &mut AttachTag<'a>) {
        walk_attach_tag(self, node);
    }

    // Blocks
    fn visit_if_block(&mut self, node: &mut IfBlock<'a>) {
        walk_if_block(self, node);
    }

    fn visit_each_block(&mut self, node: &mut EachBlock<'a>) {
        walk_each_block(self, node);
    }

    fn visit_await_block(&mut self, node: &mut AwaitBlock<'a>) {
        walk_await_block(self, node);
    }

    fn visit_key_block(&mut self, node: &mut KeyBlock<'a>) {
        walk_key_block(self, node);
    }

    fn visit_snippet_block(&mut self, node: &mut SnippetBlock<'a>) {
        walk_snippet_block(self, node);
    }

    // Elements
    fn visit_regular_element(&mut self, node: &mut RegularElement<'a>) {
        walk_regular_element(self, node);
    }

    fn visit_component(&mut self, node: &mut Component<'a>) {
        walk_component(self, node);
    }

    fn visit_svelte_element(&mut self, node: &mut SvelteElement<'a>) {
        walk_svelte_element(self, node);
    }

    fn visit_svelte_component(&mut self, node: &mut SvelteComponent<'a>) {
        walk_svelte_component(self, node);
    }

    fn visit_svelte_self(&mut self, node: &mut SvelteSelf<'a>) {
        walk_svelte_self(self, node);
    }

    fn visit_slot_element(&mut self, node: &mut SlotElement<'a>) {
        walk_slot_element(self, node);
    }

    fn visit_svelte_head(&mut self, node: &mut SvelteHead<'a>) {
        walk_svelte_head(self, node);
    }

    fn visit_svelte_body(&mut self, node: &mut SvelteBody<'a>) {
        walk_svelte_body(self, node);
    }

    fn visit_svelte_window(&mut self, node: &mut SvelteWindow<'a>) {
        walk_svelte_window(self, node);
    }

    fn visit_svelte_document(&mut self, node: &mut SvelteDocument<'a>) {
        walk_svelte_document(self, node);
    }

    fn visit_svelte_fragment(&mut self, node: &mut SvelteFragment<'a>) {
        walk_svelte_fragment(self, node);
    }

    fn visit_svelte_boundary(&mut self, node: &mut SvelteBoundary<'a>) {
        walk_svelte_boundary(self, node);
    }

    fn visit_title_element(&mut self, node: &mut TitleElement<'a>) {
        walk_title_element(self, node);
    }

    // ========================================================================
    // Attributes & Directives
    // ========================================================================

    fn visit_attribute_node(&mut self, node: &mut AttributeNode<'a>) {
        walk_attribute_node(self, node);
    }

    fn visit_attribute(&mut self, node: &mut Attribute<'a>) {
        walk_attribute(self, node);
    }

    fn visit_spread_attribute(&mut self, node: &mut SpreadAttribute<'a>) {
        walk_spread_attribute(self, node);
    }

    fn visit_bind_directive(&mut self, node: &mut BindDirective<'a>) {
        walk_bind_directive(self, node);
    }

    fn visit_class_directive(&mut self, node: &mut ClassDirective<'a>) {
        walk_class_directive(self, node);
    }

    fn visit_style_directive(&mut self, node: &mut StyleDirective<'a>) {
        walk_style_directive(self, node);
    }

    fn visit_on_directive(&mut self, node: &mut OnDirective<'a>) {
        walk_on_directive(self, node);
    }

    fn visit_transition_directive(&mut self, node: &mut TransitionDirective<'a>) {
        walk_transition_directive(self, node);
    }

    fn visit_animate_directive(&mut self, node: &mut AnimateDirective<'a>) {
        walk_animate_directive(self, node);
    }

    fn visit_use_directive(&mut self, node: &mut UseDirective<'a>) {
        walk_use_directive(self, node);
    }

    fn visit_let_directive(&mut self, node: &mut LetDirective<'a>) {
        walk_let_directive(self, node);
    }
}

// ============================================================================
// Walk functions
// ============================================================================

pub fn walk_root<'a, V: SvelteVisitor<'a> + ?Sized>(visitor: &mut V, node: &mut Root<'a>) {
    if let Some(ref mut module) = node.module {
        visitor.visit_script(module);
    }
    if let Some(ref mut instance) = node.instance {
        visitor.visit_script(instance);
    }
    visitor.visit_fragment(&mut node.fragment);
    // TODO: visit CSS
}

pub fn walk_script<'a, V: SvelteVisitor<'a> + ?Sized>(visitor: &mut V, node: &mut Script<'a>) {
    for attr in &mut node.attributes {
        visitor.visit_attribute_node(attr);
    }
    // TODO: visit JS AST via oxc visitor
}

pub fn walk_fragment<'a, V: SvelteVisitor<'a> + ?Sized>(visitor: &mut V, node: &mut Fragment<'a>) {
    for child in &mut node.nodes {
        visitor.visit_fragment_node(child);
    }
}

pub fn walk_fragment_node<'a, V: SvelteVisitor<'a> + ?Sized>(
    visitor: &mut V,
    node: &mut FragmentNode<'a>,
) {
    match node {
        FragmentNode::Text(n) => visitor.visit_text(n),
        FragmentNode::Comment(n) => visitor.visit_comment(n),
        FragmentNode::ExpressionTag(n) => visitor.visit_expression_tag(n),
        FragmentNode::HtmlTag(n) => visitor.visit_html_tag(n),
        FragmentNode::ConstTag(n) => visitor.visit_const_tag(n),
        FragmentNode::DebugTag(n) => visitor.visit_debug_tag(n),
        FragmentNode::RenderTag(n) => visitor.visit_render_tag(n),
        FragmentNode::AttachTag(n) => visitor.visit_attach_tag(n),
        FragmentNode::IfBlock(n) => visitor.visit_if_block(n),
        FragmentNode::EachBlock(n) => visitor.visit_each_block(n),
        FragmentNode::AwaitBlock(n) => visitor.visit_await_block(n),
        FragmentNode::KeyBlock(n) => visitor.visit_key_block(n),
        FragmentNode::SnippetBlock(n) => visitor.visit_snippet_block(n),
        FragmentNode::RegularElement(n) => visitor.visit_regular_element(n),
        FragmentNode::Component(n) => visitor.visit_component(n),
        FragmentNode::SvelteElement(n) => visitor.visit_svelte_element(n),
        FragmentNode::SvelteComponent(n) => visitor.visit_svelte_component(n),
        FragmentNode::SvelteSelf(n) => visitor.visit_svelte_self(n),
        FragmentNode::SlotElement(n) => visitor.visit_slot_element(n),
        FragmentNode::SvelteHead(n) => visitor.visit_svelte_head(n),
        FragmentNode::SvelteBody(n) => visitor.visit_svelte_body(n),
        FragmentNode::SvelteWindow(n) => visitor.visit_svelte_window(n),
        FragmentNode::SvelteDocument(n) => visitor.visit_svelte_document(n),
        FragmentNode::SvelteFragment(n) => visitor.visit_svelte_fragment(n),
        FragmentNode::SvelteBoundary(n) => visitor.visit_svelte_boundary(n),
        FragmentNode::TitleElement(n) => visitor.visit_title_element(n),
        FragmentNode::SvelteOptionsRaw(_) => { /* skip, handled separately */ }
    }
}

pub fn walk_attribute_node<'a, V: SvelteVisitor<'a> + ?Sized>(
    visitor: &mut V,
    node: &mut AttributeNode<'a>,
) {
    match node {
        AttributeNode::Attribute(n) => visitor.visit_attribute(n),
        AttributeNode::SpreadAttribute(n) => visitor.visit_spread_attribute(n),
        AttributeNode::BindDirective(n) => visitor.visit_bind_directive(n),
        AttributeNode::ClassDirective(n) => visitor.visit_class_directive(n),
        AttributeNode::StyleDirective(n) => visitor.visit_style_directive(n),
        AttributeNode::OnDirective(n) => visitor.visit_on_directive(n),
        AttributeNode::TransitionDirective(n) => visitor.visit_transition_directive(n),
        AttributeNode::AnimateDirective(n) => visitor.visit_animate_directive(n),
        AttributeNode::UseDirective(n) => visitor.visit_use_directive(n),
        AttributeNode::LetDirective(n) => visitor.visit_let_directive(n),
        AttributeNode::AttachTag(n) => visitor.visit_attach_tag(n),
    }
}

// Tags
pub fn walk_expression_tag<'a, V: SvelteVisitor<'a> + ?Sized>(
    _visitor: &mut V,
    _node: &mut ExpressionTag<'a>,
) {
    // TODO: visit expression via oxc visitor
}

pub fn walk_html_tag<'a, V: SvelteVisitor<'a> + ?Sized>(_visitor: &mut V, _node: &mut HtmlTag<'a>) {
    // TODO: visit expression via oxc visitor
}

pub fn walk_const_tag<'a, V: SvelteVisitor<'a> + ?Sized>(_visitor: &mut V, _node: &mut ConstTag<'a>) {
    // TODO: visit declaration via oxc visitor
}

pub fn walk_debug_tag<'a, V: SvelteVisitor<'a> + ?Sized>(_visitor: &mut V, _node: &mut DebugTag<'a>) {
    // TODO: visit identifiers via oxc visitor
}

pub fn walk_render_tag<'a, V: SvelteVisitor<'a> + ?Sized>(_visitor: &mut V, _node: &mut RenderTag<'a>) {
    // TODO: visit expression and arguments via oxc visitor
}

pub fn walk_attach_tag<'a, V: SvelteVisitor<'a> + ?Sized>(_visitor: &mut V, _node: &mut AttachTag<'a>) {
    // TODO: visit expression via oxc visitor
}

// Blocks
pub fn walk_if_block<'a, V: SvelteVisitor<'a> + ?Sized>(visitor: &mut V, node: &mut IfBlock<'a>) {
    // TODO: visit test expression via oxc visitor
    visitor.visit_fragment(&mut node.consequent);
    if let Some(ref mut alternate) = node.alternate {
        visitor.visit_fragment(alternate);
    }
}

pub fn walk_each_block<'a, V: SvelteVisitor<'a> + ?Sized>(visitor: &mut V, node: &mut EachBlock<'a>) {
    // TODO: visit expression, context, key via oxc visitor
    visitor.visit_fragment(&mut node.body);
    if let Some(ref mut fallback) = node.fallback {
        visitor.visit_fragment(fallback);
    }
}

pub fn walk_await_block<'a, V: SvelteVisitor<'a> + ?Sized>(visitor: &mut V, node: &mut AwaitBlock<'a>) {
    // TODO: visit expression, value, error via oxc visitor
    if let Some(ref mut pending) = node.pending {
        visitor.visit_fragment(pending);
    }
    if let Some(ref mut then) = node.then {
        visitor.visit_fragment(then);
    }
    if let Some(ref mut catch) = node.catch {
        visitor.visit_fragment(catch);
    }
}

pub fn walk_key_block<'a, V: SvelteVisitor<'a> + ?Sized>(visitor: &mut V, node: &mut KeyBlock<'a>) {
    // TODO: visit expression via oxc visitor
    visitor.visit_fragment(&mut node.fragment);
}

pub fn walk_snippet_block<'a, V: SvelteVisitor<'a> + ?Sized>(
    visitor: &mut V,
    node: &mut SnippetBlock<'a>,
) {
    // TODO: visit expression and parameters via oxc visitor
    visitor.visit_fragment(&mut node.body);
}

// Elements - helper for visiting attributes and fragment
fn walk_element_like<'a, V: SvelteVisitor<'a> + ?Sized>(
    visitor: &mut V,
    attributes: &mut [AttributeNode<'a>],
    fragment: &mut Fragment<'a>,
) {
    for attr in attributes {
        visitor.visit_attribute_node(attr);
    }
    visitor.visit_fragment(fragment);
}

pub fn walk_regular_element<'a, V: SvelteVisitor<'a> + ?Sized>(
    visitor: &mut V,
    node: &mut RegularElement<'a>,
) {
    walk_element_like(visitor, &mut node.attributes, &mut node.fragment);
}

pub fn walk_component<'a, V: SvelteVisitor<'a> + ?Sized>(visitor: &mut V, node: &mut Component<'a>) {
    walk_element_like(visitor, &mut node.attributes, &mut node.fragment);
}

pub fn walk_svelte_element<'a, V: SvelteVisitor<'a> + ?Sized>(
    visitor: &mut V,
    node: &mut SvelteElement<'a>,
) {
    // TODO: visit tag expression via oxc visitor
    walk_element_like(visitor, &mut node.attributes, &mut node.fragment);
}

pub fn walk_svelte_component<'a, V: SvelteVisitor<'a> + ?Sized>(
    visitor: &mut V,
    node: &mut SvelteComponent<'a>,
) {
    // TODO: visit this expression via oxc visitor
    walk_element_like(visitor, &mut node.attributes, &mut node.fragment);
}

pub fn walk_svelte_self<'a, V: SvelteVisitor<'a> + ?Sized>(visitor: &mut V, node: &mut SvelteSelf<'a>) {
    walk_element_like(visitor, &mut node.attributes, &mut node.fragment);
}

pub fn walk_slot_element<'a, V: SvelteVisitor<'a> + ?Sized>(
    visitor: &mut V,
    node: &mut SlotElement<'a>,
) {
    walk_element_like(visitor, &mut node.attributes, &mut node.fragment);
}

pub fn walk_svelte_head<'a, V: SvelteVisitor<'a> + ?Sized>(visitor: &mut V, node: &mut SvelteHead<'a>) {
    visitor.visit_fragment(&mut node.fragment);
}

pub fn walk_svelte_body<'a, V: SvelteVisitor<'a> + ?Sized>(visitor: &mut V, node: &mut SvelteBody<'a>) {
    for attr in &mut node.attributes {
        visitor.visit_attribute_node(attr);
    }
    // svelte:body has no fragment
}

pub fn walk_svelte_window<'a, V: SvelteVisitor<'a> + ?Sized>(
    visitor: &mut V,
    node: &mut SvelteWindow<'a>,
) {
    for attr in &mut node.attributes {
        visitor.visit_attribute_node(attr);
    }
    // svelte:window has no fragment
}

pub fn walk_svelte_document<'a, V: SvelteVisitor<'a> + ?Sized>(
    visitor: &mut V,
    node: &mut SvelteDocument<'a>,
) {
    for attr in &mut node.attributes {
        visitor.visit_attribute_node(attr);
    }
    // svelte:document has no fragment
}

pub fn walk_svelte_fragment<'a, V: SvelteVisitor<'a> + ?Sized>(
    visitor: &mut V,
    node: &mut SvelteFragment<'a>,
) {
    walk_element_like(visitor, &mut node.attributes, &mut node.fragment);
}

pub fn walk_svelte_boundary<'a, V: SvelteVisitor<'a> + ?Sized>(
    visitor: &mut V,
    node: &mut SvelteBoundary<'a>,
) {
    walk_element_like(visitor, &mut node.attributes, &mut node.fragment);
}

pub fn walk_title_element<'a, V: SvelteVisitor<'a> + ?Sized>(
    visitor: &mut V,
    node: &mut TitleElement<'a>,
) {
    walk_element_like(visitor, &mut node.attributes, &mut node.fragment);
}

// Attributes & Directives
pub fn walk_attribute<'a, V: SvelteVisitor<'a> + ?Sized>(_visitor: &mut V, _node: &mut Attribute<'a>) {
    // TODO: visit value expressions if any
}

pub fn walk_spread_attribute<'a, V: SvelteVisitor<'a> + ?Sized>(
    _visitor: &mut V,
    _node: &mut SpreadAttribute<'a>,
) {
    // TODO: visit expression via oxc visitor
}

pub fn walk_bind_directive<'a, V: SvelteVisitor<'a> + ?Sized>(
    _visitor: &mut V,
    _node: &mut BindDirective<'a>,
) {
    // TODO: visit expression via oxc visitor
}

pub fn walk_class_directive<'a, V: SvelteVisitor<'a> + ?Sized>(
    _visitor: &mut V,
    _node: &mut ClassDirective<'a>,
) {
    // TODO: visit expression via oxc visitor
}

pub fn walk_style_directive<'a, V: SvelteVisitor<'a> + ?Sized>(
    _visitor: &mut V,
    _node: &mut StyleDirective<'a>,
) {
    // TODO: visit value if any
}

pub fn walk_on_directive<'a, V: SvelteVisitor<'a> + ?Sized>(
    _visitor: &mut V,
    _node: &mut OnDirective<'a>,
) {
    // TODO: visit expression via oxc visitor
}

pub fn walk_transition_directive<'a, V: SvelteVisitor<'a> + ?Sized>(
    _visitor: &mut V,
    _node: &mut TransitionDirective<'a>,
) {
    // TODO: visit expression via oxc visitor
}

pub fn walk_animate_directive<'a, V: SvelteVisitor<'a> + ?Sized>(
    _visitor: &mut V,
    _node: &mut AnimateDirective<'a>,
) {
    // TODO: visit expression via oxc visitor
}

pub fn walk_use_directive<'a, V: SvelteVisitor<'a> + ?Sized>(
    _visitor: &mut V,
    _node: &mut UseDirective<'a>,
) {
    // TODO: visit expression via oxc visitor
}

pub fn walk_let_directive<'a, V: SvelteVisitor<'a> + ?Sized>(
    _visitor: &mut V,
    _node: &mut LetDirective<'a>,
) {
    // TODO: visit expression via oxc visitor
}
