use oxc_ast::ast::Expression;

use crate::common::Span;
use crate::template::tag::{ExpressionTag, TextOrExpressionTag};

#[derive(Debug)]
pub struct BindDirective<'a> {
    pub span: Span,
    pub name: &'a str,
    pub expression: Expression<'a>,
}

#[derive(Debug)]
pub struct ClassDirective<'a> {
    pub span: Span,
    pub name: &'a str,
    pub expression: Expression<'a>,
}

#[derive(Debug)]
pub struct StyleDirective<'a> {
    pub span: Span,
    pub name: &'a str,
    pub value: StyleDirectiveValue<'a>,
    pub modifiers: Vec<StyleModifier>,
}

#[derive(Debug)]
pub enum StyleDirectiveValue<'a> {
    True,
    ExpressionTag(ExpressionTag<'a>),
    Sequence(Vec<TextOrExpressionTag<'a>>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleModifier {
    Important,
}

#[derive(Debug)]
pub struct OnDirective<'a> {
    pub span: Span,
    pub name: &'a str,
    pub expression: Option<Expression<'a>>,
    pub modifiers: Vec<EventModifier>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventModifier {
    Capture,
    Nonpassive,
    Once,
    Passive,
    PreventDefault,
    Self_,
    StopImmediatePropagation,
    StopPropagation,
    Trusted,
}

#[derive(Debug)]
pub struct TransitionDirective<'a> {
    pub span: Span,
    pub name: &'a str,
    pub expression: Option<Expression<'a>>,
    pub modifiers: Vec<TransitionModifier>,
    pub intro: bool,
    pub outro: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionModifier {
    Local,
    Global,
}

#[derive(Debug)]
pub struct AnimateDirective<'a> {
    pub span: Span,
    pub name: &'a str,
    pub expression: Option<Expression<'a>>,
}

#[derive(Debug)]
pub struct UseDirective<'a> {
    pub span: Span,
    pub name: &'a str,
    pub expression: Option<Expression<'a>>,
}

#[derive(Debug)]
pub struct LetDirective<'a> {
    pub span: Span,
    pub name: &'a str,
    pub expression: Option<Expression<'a>>,
}
