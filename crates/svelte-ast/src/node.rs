use serde::Serialize;

use crate::span::Span;
use crate::text::{Text, Comment};
use crate::tags::{ExpressionTag, HtmlTag, ConstTag, DebugTag, RenderTag, AttachTag};
use crate::blocks::{IfBlock, EachBlock, AwaitBlock, KeyBlock, SnippetBlock};
use crate::elements::{
    RegularElement, Component, SvelteElement, SvelteComponent, SvelteSelf,
    SlotElement, SvelteHead, SvelteBody, SvelteWindow, SvelteDocument,
    SvelteFragment, SvelteBoundary, TitleElement, SvelteOptionsRaw,
};
use crate::attributes::{
    Attribute, SpreadAttribute, BindDirective, ClassDirective, StyleDirective,
    OnDirective, TransitionDirective, AnimateDirective, UseDirective, LetDirective,
};
use crate::root::{Root, Fragment, Script};
use crate::css::{
    StyleSheet, CssRule, CssAtrule, CssBlock, CssDeclaration,
    SelectorList, ComplexSelector, RelativeSelector, CssCombinator,
    TypeSelector, IdSelector, ClassSelector, AttributeSelector,
    PseudoClassSelector, PseudoElementSelector, NestingSelector,
    Percentage, Nth,
};

/// All AST nodes share this structure: a source span + a kind discriminator.
#[derive(Debug, Clone, Serialize)]
pub struct AstNode {
    pub span: Span,
    pub kind: NodeKind,
}

impl AstNode {
    pub fn new(span: Span, kind: NodeKind) -> Self {
        Self { span, kind }
    }
}

/// Discriminated union of all possible AST node types.
#[derive(Debug, Clone, Serialize)]
pub enum NodeKind {
    // ── Root & Structure ──
    Root(Root),
    Fragment(Fragment),
    Script(Script),

    // ── Text ──
    Text(Text),
    Comment(Comment),

    // ── Tags ──
    ExpressionTag(ExpressionTag),
    HtmlTag(HtmlTag),
    ConstTag(ConstTag),
    DebugTag(DebugTag),
    RenderTag(RenderTag),
    AttachTag(AttachTag),

    // ── Blocks ──
    IfBlock(IfBlock),
    EachBlock(EachBlock),
    AwaitBlock(AwaitBlock),
    KeyBlock(KeyBlock),
    SnippetBlock(SnippetBlock),

    // ── Elements ──
    RegularElement(RegularElement),
    Component(Component),
    SvelteElement(SvelteElement),
    SvelteComponent(SvelteComponent),
    SvelteSelf(SvelteSelf),
    SlotElement(SlotElement),
    SvelteHead(SvelteHead),
    SvelteBody(SvelteBody),
    SvelteWindow(SvelteWindow),
    SvelteDocument(SvelteDocument),
    SvelteFragment(SvelteFragment),
    SvelteBoundary(SvelteBoundary),
    TitleElement(TitleElement),
    SvelteOptions(SvelteOptionsRaw),

    // ── Attributes & Directives ──
    Attribute(Attribute),
    SpreadAttribute(SpreadAttribute),
    BindDirective(BindDirective),
    ClassDirective(ClassDirective),
    StyleDirective(StyleDirective),
    OnDirective(OnDirective),
    TransitionDirective(TransitionDirective),
    AnimateDirective(AnimateDirective),
    UseDirective(UseDirective),
    LetDirective(LetDirective),

    // ── CSS ──
    StyleSheet(StyleSheet),
    CssRule(CssRule),
    CssAtrule(CssAtrule),
    CssBlock(CssBlock),
    CssDeclaration(CssDeclaration),
    SelectorList(SelectorList),
    ComplexSelector(ComplexSelector),
    RelativeSelector(RelativeSelector),
    CssCombinator(CssCombinator),
    TypeSelector(TypeSelector),
    IdSelector(IdSelector),
    ClassSelector(ClassSelector),
    AttributeSelector(AttributeSelector),
    PseudoClassSelector(PseudoClassSelector),
    PseudoElementSelector(PseudoElementSelector),
    NestingSelector(NestingSelector),
    Percentage(Percentage),
    Nth(Nth),
}
