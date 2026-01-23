use serde::Serialize;

use crate::attributes::{
    AnimateDirective, Attribute, BindDirective, ClassDirective, LetDirective, OnDirective,
    SpreadAttribute, StyleDirective, TransitionDirective, UseDirective,
};
use crate::blocks::{AwaitBlock, EachBlock, IfBlock, KeyBlock, SnippetBlock};
use crate::css::{
    AttributeSelector, ClassSelector, CssAtrule, CssDeclaration, CssRule, IdSelector,
    NestingSelector, Nth, Percentage, PseudoClassSelector, PseudoElementSelector, TypeSelector,
};
use crate::elements::{
    Component, RegularElement, SlotElement, SvelteBody, SvelteBoundary, SvelteComponent,
    SvelteDocument, SvelteElement, SvelteFragment, SvelteHead, SvelteOptionsRaw, SvelteSelf,
    SvelteWindow, TitleElement,
};
use crate::tags::{AttachTag, ConstTag, DebugTag, ExpressionTag, HtmlTag, RenderTag};
use crate::text::{Comment, Text};

/*
 * type FragmentNode = Text | Tag | ElementLike | Block | Comment;
 */
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum FragmentNode {
    // Text
    Text(Text),
    Comment(Comment),

    // Tags
    ExpressionTag(ExpressionTag),
    HtmlTag(HtmlTag),
    ConstTag(ConstTag),
    DebugTag(DebugTag),
    RenderTag(RenderTag),
    AttachTag(AttachTag),

    // Blocks
    IfBlock(IfBlock),
    EachBlock(EachBlock),
    AwaitBlock(AwaitBlock),
    KeyBlock(KeyBlock),
    SnippetBlock(SnippetBlock),

    // Elements
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
    SvelteOptionsRaw(SvelteOptionsRaw),
}

/*
 * type AttributeNode = Attribute | SpreadAttribute | Directive;
 */
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum AttributeNode {
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
}

/*
 * type StyleSheetChild = Atrule | Rule;
 */
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum StyleSheetChild {
    Rule(CssRule),
    Atrule(CssAtrule),
}

/*
 * type CssBlockChild = Declaration | Rule | Atrule;
 */
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum CssBlockChild {
    Declaration(CssDeclaration),
    Rule(CssRule),
    Atrule(CssAtrule),
}

/*
 * type SimpleSelector = TypeSelector | IdSelector | ClassSelector
 *   | AttributeSelector | PseudoElementSelector | PseudoClassSelector
 *   | Percentage | Nth | NestingSelector;
 */
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum SimpleSelector {
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
