use oxc_ast::ast::Expression;

use crate::common::Span;
use crate::template::attribute::AttributeNode;
use crate::template::root::Fragment;

#[derive(Debug)]
pub struct RegularElement<'a> {
    pub span: Span,
    pub name: &'a str,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

#[derive(Debug)]
pub struct Component<'a> {
    pub span: Span,
    pub name: &'a str,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

#[derive(Debug)]
pub struct SvelteComponent<'a> {
    pub span: Span,
    pub name: &'a str,
    pub expression: Expression<'a>,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

#[derive(Debug)]
pub struct SvelteElement<'a> {
    pub span: Span,
    pub name: &'a str,
    pub tag: Expression<'a>,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

#[derive(Debug)]
pub struct SvelteSelf<'a> {
    pub span: Span,
    pub name: &'a str,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

// --- Simple svelte elements (same shape) ---

#[derive(Debug)]
pub struct SvelteHead<'a> {
    pub span: Span,
    pub name: &'a str,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

#[derive(Debug)]
pub struct SvelteBody<'a> {
    pub span: Span,
    pub name: &'a str,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

#[derive(Debug)]
pub struct SvelteWindow<'a> {
    pub span: Span,
    pub name: &'a str,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

#[derive(Debug)]
pub struct SvelteDocument<'a> {
    pub span: Span,
    pub name: &'a str,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

#[derive(Debug)]
pub struct SvelteFragment<'a> {
    pub span: Span,
    pub name: &'a str,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

#[derive(Debug)]
pub struct SvelteBoundary<'a> {
    pub span: Span,
    pub name: &'a str,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

#[derive(Debug)]
pub struct TitleElement<'a> {
    pub span: Span,
    pub name: &'a str,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

#[derive(Debug)]
pub struct SlotElement<'a> {
    pub span: Span,
    pub name: &'a str,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

#[derive(Debug)]
pub struct SvelteOptionsRaw<'a> {
    pub span: Span,
    pub name: &'a str,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}
