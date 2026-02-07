use crate::common::Span;
use crate::css::selector::SelectorList;
use crate::template::attribute::Attribute;
use crate::template::tag::Comment;

#[derive(Debug)]
pub struct StyleSheet<'a> {
    pub span: Span,
    pub attributes: Vec<Attribute<'a>>,
    pub children: Vec<StyleSheetChild<'a>>,
    pub content_start: u32,
    pub content_end: u32,
    pub content_styles: &'a str,
    pub content_comment: Option<Comment<'a>>,
}

#[derive(Debug)]
pub enum StyleSheetChild<'a> {
    Rule(CssRule<'a>),
    Atrule(CssAtrule<'a>),
}

#[derive(Debug)]
pub struct CssRule<'a> {
    pub span: Span,
    pub prelude: SelectorList<'a>,
    pub block: CssBlock<'a>,
    // metadata (analysis phase)
    pub parent_rule: Option<usize>,
    pub has_local_selectors: bool,
    pub has_global_selectors: bool,
    pub is_global_block: bool,
}

#[derive(Debug)]
pub struct CssAtrule<'a> {
    pub span: Span,
    pub name: &'a str,
    pub prelude: &'a str,
    pub block: Option<CssBlock<'a>>,
}

#[derive(Debug)]
pub struct CssBlock<'a> {
    pub span: Span,
    pub children: Vec<CssBlockChild<'a>>,
}

#[derive(Debug)]
pub enum CssBlockChild<'a> {
    Declaration(CssDeclaration<'a>),
    Rule(CssRule<'a>),
    Atrule(CssAtrule<'a>),
}

#[derive(Debug)]
pub struct CssDeclaration<'a> {
    pub span: Span,
    pub property: &'a str,
    pub value: &'a str,
}
