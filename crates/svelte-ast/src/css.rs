use serde::ser::SerializeMap;
use serde::Serialize;

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
    pub content: CssContent<'a>,
}

impl Serialize for StyleSheet<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "StyleSheet")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("attributes", &self.attributes)?;
        map.serialize_entry("children", &self.children)?;
        map.serialize_entry("content", &self.content)?;
        map.end()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CssContent<'a> {
    pub start: u32,
    pub end: u32,
    pub styles: &'a str,
    pub comment: Option<Comment<'a>>,
}

/*
 * type StyleSheetChild = Atrule | Rule;
 */
#[derive(Debug, Clone)]
pub enum StyleSheetChild {
    Rule(CssRule),
    Atrule(CssAtrule),
}

impl Serialize for StyleSheetChild {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Rule(n) => n.serialize(s),
            Self::Atrule(n) => n.serialize(s),
        }
    }
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

impl Serialize for CssBlockChild {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Declaration(n) => n.serialize(s),
            Self::Rule(n) => n.serialize(s),
            Self::Atrule(n) => n.serialize(s),
        }
    }
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

impl Serialize for SimpleSelector {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::TypeSelector(n) => n.serialize(s),
            Self::IdSelector(n) => n.serialize(s),
            Self::ClassSelector(n) => n.serialize(s),
            Self::AttributeSelector(n) => n.serialize(s),
            Self::PseudoClassSelector(n) => n.serialize(s),
            Self::PseudoElementSelector(n) => n.serialize(s),
            Self::NestingSelector(n) => n.serialize(s),
            Self::Percentage(n) => n.serialize(s),
            Self::Nth(n) => n.serialize(s),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CssRule {
    pub span: Span,
    pub prelude: SelectorList,
    pub block: CssBlock,
}

#[derive(Debug, Clone, Serialize)]
pub struct CssAtrule {
    pub span: Span,
    pub name: String,
    pub prelude: String,
    pub block: Option<CssBlock>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CssBlock {
    pub span: Span,
    pub children: Vec<CssBlockChild>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CssDeclaration {
    pub span: Span,
    pub property: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SelectorList {
    pub span: Span,
    pub children: Vec<ComplexSelector>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComplexSelector {
    pub span: Span,
    pub children: Vec<RelativeSelector>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RelativeSelector {
    pub span: Span,
    pub combinator: Option<CssCombinator>,
    pub selectors: Vec<SimpleSelector>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CssCombinator {
    pub span: Span,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TypeSelector {
    pub span: Span,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IdSelector {
    pub span: Span,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClassSelector {
    pub span: Span,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AttributeSelector {
    pub span: Span,
    pub name: String,
    pub matcher: Option<String>,
    pub value: Option<String>,
    pub flags: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PseudoClassSelector {
    pub span: Span,
    pub name: String,
    pub args: Option<Box<SelectorList>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PseudoElementSelector {
    pub span: Span,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NestingSelector {
    pub span: Span,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Percentage {
    pub span: Span,
    pub value: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Nth {
    pub span: Span,
    pub value: String,
}
