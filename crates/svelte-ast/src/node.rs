use serde::Serialize;

use crate::attributes::{
    AnimateDirective, Attribute, BindDirective, ClassDirective, LetDirective, OnDirective,
    SpreadAttribute, StyleDirective, TransitionDirective, UseDirective,
};
use crate::blocks::{AwaitBlock, EachBlock, IfBlock, KeyBlock, SnippetBlock};
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
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum FragmentNode<'a> {
    // Text
    Text(Text<'a>),
    Comment(Comment<'a>),

    // Tags
    ExpressionTag(ExpressionTag<'a>),
    HtmlTag(HtmlTag<'a>),
    ConstTag(ConstTag<'a>),
    DebugTag(DebugTag<'a>),
    RenderTag(RenderTag<'a>),
    AttachTag(AttachTag<'a>),

    // Blocks
    IfBlock(IfBlock<'a>),
    EachBlock(EachBlock<'a>),
    AwaitBlock(AwaitBlock<'a>),
    KeyBlock(KeyBlock<'a>),
    SnippetBlock(SnippetBlock<'a>),

    // Elements
    RegularElement(RegularElement<'a>),
    Component(Component<'a>),
    SvelteElement(SvelteElement<'a>),
    SvelteComponent(SvelteComponent<'a>),
    SvelteSelf(SvelteSelf<'a>),
    SlotElement(SlotElement<'a>),
    SvelteHead(SvelteHead<'a>),
    SvelteBody(SvelteBody<'a>),
    SvelteWindow(SvelteWindow<'a>),
    SvelteDocument(SvelteDocument<'a>),
    SvelteFragment(SvelteFragment<'a>),
    SvelteBoundary(SvelteBoundary<'a>),
    TitleElement(TitleElement<'a>),
    SvelteOptionsRaw(SvelteOptionsRaw<'a>),
}

/*
 * type AttributeNode = Attribute | SpreadAttribute | Directive | AttachTag;
 */
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum AttributeNode<'a> {
    Attribute(Attribute<'a>),
    SpreadAttribute(SpreadAttribute<'a>),
    BindDirective(BindDirective<'a>),
    ClassDirective(ClassDirective<'a>),
    StyleDirective(StyleDirective<'a>),
    OnDirective(OnDirective<'a>),
    TransitionDirective(TransitionDirective<'a>),
    AnimateDirective(AnimateDirective<'a>),
    UseDirective(UseDirective<'a>),
    LetDirective(LetDirective<'a>),
    AttachTag(AttachTag<'a>),
}
