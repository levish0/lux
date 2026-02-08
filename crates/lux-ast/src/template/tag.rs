use oxc_ast::ast::{Expression, IdentifierReference};

use crate::common::Span;

#[derive(Debug)]
pub struct Text<'a> {
    pub span: Span,
    pub data: &'a str,
    pub raw: &'a str,
}

#[derive(Debug)]
pub struct Comment<'a> {
    pub span: Span,
    pub data: &'a str,
}

#[derive(Debug)]
pub struct JsComment<'a> {
    pub span: Span,
    pub kind: JsCommentKind,
    pub value: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsCommentKind {
    Line,
    Block,
}

#[derive(Debug)]
pub struct ExpressionTag<'a> {
    pub span: Span,
    pub expression: Expression<'a>,
}

#[derive(Debug)]
pub struct HtmlTag<'a> {
    pub span: Span,
    pub expression: Expression<'a>,
}

#[derive(Debug)]
pub struct ConstTag<'a> {
    pub span: Span,
    pub declaration: ConstDeclaration<'a>,
}

#[derive(Debug)]
pub struct ConstDeclaration<'a> {
    pub span: Span,
    pub id: Expression<'a>,
    pub init: Expression<'a>,
}

#[derive(Debug)]
pub struct DebugTag<'a> {
    pub span: Span,
    pub identifiers: Vec<IdentifierReference<'a>>,
}

#[derive(Debug)]
pub struct RenderTag<'a> {
    pub span: Span,
    pub expression: Expression<'a>,
}

#[derive(Debug)]
pub struct AttachTag<'a> {
    pub span: Span,
    pub expression: Expression<'a>,
}

/// Either Text or ExpressionTag — used in attribute value sequences.
#[derive(Debug)]
pub enum TextOrExpressionTag<'a> {
    Text(Text<'a>),
    ExpressionTag(ExpressionTag<'a>),
}
