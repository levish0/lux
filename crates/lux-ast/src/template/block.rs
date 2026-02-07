use oxc_ast::ast::{Expression, IdentifierReference};

use crate::common::Span;
use crate::metadata::{EachBlockMetadata, ExpressionMetadata, SnippetBlockMetadata};
use crate::template::root::Fragment;

#[derive(Debug)]
pub struct IfBlock<'a> {
    pub span: Span,
    pub elseif: bool,
    pub test: Expression<'a>,
    pub consequent: Fragment<'a>,
    pub alternate: Option<Fragment<'a>>,
    pub metadata: Option<ExpressionMetadata>,
}

#[derive(Debug)]
pub struct EachBlock<'a> {
    pub span: Span,
    pub expression: Expression<'a>,
    pub context: Option<Expression<'a>>,
    pub body: Fragment<'a>,
    pub fallback: Option<Fragment<'a>>,
    pub index: Option<&'a str>,
    pub key: Option<Expression<'a>>,
    pub metadata: Option<EachBlockMetadata>,
}

#[derive(Debug)]
pub struct AwaitBlock<'a> {
    pub span: Span,
    pub expression: Expression<'a>,
    pub value: Option<Expression<'a>>,
    pub error: Option<Expression<'a>>,
    pub pending: Option<Fragment<'a>>,
    pub then: Option<Fragment<'a>>,
    pub catch: Option<Fragment<'a>>,
    pub metadata: Option<ExpressionMetadata>,
}

#[derive(Debug)]
pub struct KeyBlock<'a> {
    pub span: Span,
    pub expression: Expression<'a>,
    pub fragment: Fragment<'a>,
    pub metadata: Option<ExpressionMetadata>,
}

#[derive(Debug)]
pub struct SnippetBlock<'a> {
    pub span: Span,
    pub expression: IdentifierReference<'a>,
    pub parameters: Vec<Expression<'a>>,
    pub body: Fragment<'a>,
    pub metadata: Option<SnippetBlockMetadata>,
}
