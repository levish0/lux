use oxc_ast::ast::Expression;

use crate::common::Span;
use crate::template::directive::{
    AnimateDirective, BindDirective, ClassDirective, LetDirective, OnDirective, StyleDirective,
    TransitionDirective, UseDirective,
};
use crate::template::tag::{AttachTag, ExpressionTag, TextOrExpressionTag};

/// Union of all attribute-position nodes.
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

#[derive(Debug)]
pub struct Attribute<'a> {
    pub span: Span,
    pub name: &'a str,
    pub value: AttributeValue<'a>,
}

#[derive(Debug)]
pub enum AttributeValue<'a> {
    /// Boolean attribute (e.g., `disabled`).
    True,
    /// Single expression (e.g., `{expr}`).
    ExpressionTag(ExpressionTag<'a>),
    /// Quoted sequence (e.g., `"text{expr}text"`).
    Sequence(Vec<TextOrExpressionTag<'a>>),
}

#[derive(Debug)]
pub struct SpreadAttribute<'a> {
    pub span: Span,
    pub expression: Expression<'a>,
}
