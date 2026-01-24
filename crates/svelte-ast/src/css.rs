use crate::attributes::Attribute;
use crate::span::Span;
use crate::text::Comment;

/*
 * interface StyleSheet extends BaseNode {
 *   type: 'StyleSheet';
 *   attributes: any[];
 *   children: Array<Atrule | Rule>;
 *   content: { start: number; end: number; styles: string; comment: Comment | null; };
 * }
 */
#[derive(Debug)]
pub struct StyleSheet<'a> {
    pub span: Span,
    pub attributes: Vec<Attribute<'a>>,
    pub children: Vec<StyleSheetChild>,
    pub content: CssContent,
}

#[derive(Debug, Clone)]
pub struct CssContent {
    pub start: u32,
    pub end: u32,
    pub styles: String,
    pub comment: Option<Comment>,
}

/*
 * type StyleSheetChild = Atrule | Rule;
 */
#[derive(Debug, Clone)]
pub enum StyleSheetChild {
    Rule(CssRule),
    Atrule(CssAtrule),
}

/*
 * type CssBlockChild = Declaration | Rule | Atrule;
 */
#[derive(Debug, Clone)]
pub enum CssBlockChild {
    Declaration(CssDeclaration),
    Rule(CssRule),
    Atrule(CssAtrule),
}

/*
 * type SimpleSelector = TypeSelector | IdSelector | ClassSelector
 *   | AttributeSelector | PseudoElementSelector | PseudoClassSelector
 *   | Percentage | Nth | NestingSelector;
 */
#[derive(Debug, Clone)]
pub enum SimpleSelector {
    TypeSelector(TypeSelector),
    IdSelector(IdSelector),
    ClassSelector(ClassSelector),
    AttributeSelector(AttributeSelector),
    PseudoClassSelector(PseudoClassSelector),
    PseudoElementSelector(PseudoElementSelector),
    NestingSelector(NestingSelector),
    Percentage(Percentage),
    Nth(Nth),
}

#[derive(Debug, Clone)]
pub struct CssRule {
    pub span: Span,
    pub prelude: SelectorList,
    pub block: CssBlock,
}

#[derive(Debug, Clone)]
pub struct CssAtrule {
    pub span: Span,
    pub name: String,
    pub prelude: String,
    pub block: Option<CssBlock>,
}

#[derive(Debug, Clone)]
pub struct CssBlock {
    pub span: Span,
    pub children: Vec<CssBlockChild>,
}

#[derive(Debug, Clone)]
pub struct CssDeclaration {
    pub span: Span,
    pub property: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct SelectorList {
    pub span: Span,
    pub children: Vec<ComplexSelector>,
}

#[derive(Debug, Clone)]
pub struct ComplexSelector {
    pub span: Span,
    pub children: Vec<RelativeSelector>,
}

#[derive(Debug, Clone)]
pub struct RelativeSelector {
    pub span: Span,
    pub combinator: Option<CssCombinator>,
    pub selectors: Vec<SimpleSelector>,
}

#[derive(Debug, Clone)]
pub struct CssCombinator {
    pub span: Span,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct TypeSelector {
    pub span: Span,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct IdSelector {
    pub span: Span,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct ClassSelector {
    pub span: Span,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct AttributeSelector {
    pub span: Span,
    pub name: String,
    pub matcher: Option<String>,
    pub value: Option<String>,
    pub flags: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PseudoClassSelector {
    pub span: Span,
    pub name: String,
    pub args: Option<Box<SelectorList>>,
}

#[derive(Debug, Clone)]
pub struct PseudoElementSelector {
    pub span: Span,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct NestingSelector {
    pub span: Span,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Percentage {
    pub span: Span,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct Nth {
    pub span: Span,
    pub value: String,
}
