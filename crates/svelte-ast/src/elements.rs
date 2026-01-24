use oxc_ast::ast::Expression;

use crate::node::AttributeNode;
use crate::root::Fragment;
use crate::span::{SourceLocation, Span};

/*
 * interface BaseElement extends BaseNode {
 *   name: string;
 *   name_loc: SourceLocation;
 *   attributes: Array<Attribute | SpreadAttribute | Directive | AttachTag>;
 *   fragment: Fragment;
 * }
 */

#[derive(Debug)]
pub struct RegularElement<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

#[derive(Debug)]
pub struct Component<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

#[derive(Debug)]
pub struct SvelteElement<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: SourceLocation,
    pub tag: Expression<'a>,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

#[derive(Debug)]
pub struct SvelteComponent<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: SourceLocation,
    pub expression: Expression<'a>,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

#[derive(Debug)]
pub struct SvelteSelf<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

#[derive(Debug)]
pub struct SlotElement<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

#[derive(Debug)]
pub struct SvelteHead<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

#[derive(Debug)]
pub struct SvelteBody<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

#[derive(Debug)]
pub struct SvelteWindow<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

#[derive(Debug)]
pub struct SvelteDocument<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

#[derive(Debug)]
pub struct SvelteFragment<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

#[derive(Debug)]
pub struct SvelteBoundary<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

#[derive(Debug)]
pub struct TitleElement<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

#[derive(Debug)]
pub struct SvelteOptionsRaw<'a> {
    pub span: Span,
    pub name: &'a str,
    pub name_loc: SourceLocation,
    pub attributes: Vec<AttributeNode<'a>>,
    pub fragment: Fragment<'a>,
}

