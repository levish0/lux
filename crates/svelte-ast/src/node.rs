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
#[derive(Debug)]
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

impl Serialize for FragmentNode<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Text(n) => n.serialize(s),
            Self::Comment(n) => n.serialize(s),
            Self::ExpressionTag(n) => n.serialize(s),
            Self::HtmlTag(n) => n.serialize(s),
            Self::ConstTag(n) => n.serialize(s),
            Self::DebugTag(n) => n.serialize(s),
            Self::RenderTag(n) => n.serialize(s),
            Self::AttachTag(n) => n.serialize(s),
            Self::IfBlock(n) => n.serialize(s),
            Self::EachBlock(n) => n.serialize(s),
            Self::AwaitBlock(n) => n.serialize(s),
            Self::KeyBlock(n) => n.serialize(s),
            Self::SnippetBlock(n) => n.serialize(s),
            Self::RegularElement(n) => n.serialize(s),
            Self::Component(n) => n.serialize(s),
            Self::SvelteElement(n) => n.serialize(s),
            Self::SvelteComponent(n) => n.serialize(s),
            Self::SvelteSelf(n) => n.serialize(s),
            Self::SlotElement(n) => n.serialize(s),
            Self::SvelteHead(n) => n.serialize(s),
            Self::SvelteBody(n) => n.serialize(s),
            Self::SvelteWindow(n) => n.serialize(s),
            Self::SvelteDocument(n) => n.serialize(s),
            Self::SvelteFragment(n) => n.serialize(s),
            Self::SvelteBoundary(n) => n.serialize(s),
            Self::TitleElement(n) => n.serialize(s),
            Self::SvelteOptionsRaw(n) => n.serialize(s),
        }
    }
}

/*
 * type AttributeNode = Attribute | SpreadAttribute | Directive | AttachTag;
 */
#[derive(Debug)]
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

impl Serialize for AttributeNode<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Attribute(n) => n.serialize(s),
            Self::SpreadAttribute(n) => n.serialize(s),
            Self::BindDirective(n) => n.serialize(s),
            Self::ClassDirective(n) => n.serialize(s),
            Self::StyleDirective(n) => n.serialize(s),
            Self::OnDirective(n) => n.serialize(s),
            Self::TransitionDirective(n) => n.serialize(s),
            Self::AnimateDirective(n) => n.serialize(s),
            Self::UseDirective(n) => n.serialize(s),
            Self::LetDirective(n) => n.serialize(s),
            Self::AttachTag(n) => n.serialize(s),
        }
    }
}
