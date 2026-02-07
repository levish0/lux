use oxc_ast::ast::{ArrowFunctionExpression, ObjectExpression, Program};

use crate::common::Span;
use crate::css::StyleSheet;
use crate::template::attribute::Attribute;
use crate::template::block::{AwaitBlock, EachBlock, IfBlock, KeyBlock, SnippetBlock};
use crate::template::element::{
    Component, RegularElement, SlotElement, SvelteBody, SvelteBoundary, SvelteComponent,
    SvelteDocument, SvelteElement, SvelteFragment, SvelteHead, SvelteOptionsRaw, SvelteSelf,
    SvelteWindow, TitleElement,
};
use crate::template::tag::{
    AttachTag, Comment, ConstTag, DebugTag, ExpressionTag, HtmlTag, JsComment, RenderTag, Text,
};

#[derive(Debug)]
pub struct Root<'a> {
    pub span: Span,
    pub options: Option<SvelteOptions<'a>>,
    pub fragment: Fragment<'a>,
    pub css: Option<StyleSheet<'a>>,
    pub instance: Option<Script<'a>>,
    pub module: Option<Script<'a>>,
    pub comments: Vec<JsComment<'a>>,
    pub ts: bool,
}

#[derive(Debug)]
pub struct Fragment<'a> {
    pub nodes: Vec<FragmentNode<'a>>,
    pub transparent: bool,
    pub dynamic: bool,
}

#[derive(Debug)]
pub enum FragmentNode<'a> {
    // Text and tags
    Text(Text<'a>),
    ExpressionTag(ExpressionTag<'a>),
    HtmlTag(HtmlTag<'a>),
    ConstTag(ConstTag<'a>),
    DebugTag(DebugTag<'a>),
    RenderTag(RenderTag<'a>),
    AttachTag(AttachTag<'a>),
    Comment(Comment<'a>),
    // Elements
    RegularElement(RegularElement<'a>),
    Component(Component<'a>),
    SvelteElement(SvelteElement<'a>),
    SvelteComponent(SvelteComponent<'a>),
    SvelteSelf(SvelteSelf<'a>),
    SvelteFragment(SvelteFragment<'a>),
    SvelteHead(SvelteHead<'a>),
    SvelteBody(SvelteBody<'a>),
    SvelteWindow(SvelteWindow<'a>),
    SvelteDocument(SvelteDocument<'a>),
    SvelteBoundary(SvelteBoundary<'a>),
    SlotElement(SlotElement<'a>),
    TitleElement(TitleElement<'a>),
    SvelteOptionsRaw(SvelteOptionsRaw<'a>),
    // Blocks
    IfBlock(IfBlock<'a>),
    EachBlock(EachBlock<'a>),
    AwaitBlock(AwaitBlock<'a>),
    KeyBlock(KeyBlock<'a>),
    SnippetBlock(SnippetBlock<'a>),
}

#[derive(Debug)]
pub struct Script<'a> {
    pub span: Span,
    pub context: ScriptContext,
    pub content: Program<'a>,
    pub attributes: Vec<Attribute<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptContext {
    Default,
    Module,
}

#[derive(Debug)]
pub struct SvelteOptions<'a> {
    pub span: Span,
    pub runes: Option<bool>,
    pub immutable: Option<bool>,
    pub accessors: Option<bool>,
    pub preserve_whitespace: Option<bool>,
    pub namespace: Option<Namespace>,
    pub css: Option<CssOption>,
    pub custom_element: Option<CustomElementOptions<'a>>,
    pub attributes: Vec<Attribute<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Namespace {
    Html,
    Svg,
    Mathml,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CssOption {
    Injected,
}

#[derive(Debug)]
pub struct CustomElementOptions<'a> {
    pub tag: Option<&'a str>,
    pub shadow: Option<&'a str>,
    pub props: Option<ObjectExpression<'a>>,
    pub extend: Option<ArrowFunctionExpression<'a>>,
}
