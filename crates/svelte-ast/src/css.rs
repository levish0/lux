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
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum StyleSheetChild {
    Rule(CssRule),
    Atrule(CssAtrule),
}

/*
 * type CssBlockChild = Declaration | Rule | Atrule;
 */
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
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
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
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

macro_rules! impl_css_serialize {
    ($ty:ident, $type_str:expr, [$($field:ident),* $(,)?]) => {
        impl Serialize for $ty {
            fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                let mut map = s.serialize_map(None)?;
                map.serialize_entry("type", $type_str)?;
                map.serialize_entry("start", &self.span.start)?;
                map.serialize_entry("end", &self.span.end)?;
                $(map.serialize_entry(stringify!($field), &self.$field)?;)*
                map.end()
            }
        }
    };
}

#[derive(Debug, Clone)]
pub struct CssRule {
    pub span: Span,
    pub prelude: SelectorList,
    pub block: CssBlock,
}

impl_css_serialize!(CssRule, "Rule", [prelude, block]);

#[derive(Debug, Clone)]
pub struct CssAtrule {
    pub span: Span,
    pub name: String,
    pub prelude: String,
    pub block: Option<CssBlock>,
}

impl_css_serialize!(CssAtrule, "Atrule", [name, prelude, block]);

#[derive(Debug, Clone)]
pub struct CssBlock {
    pub span: Span,
    pub children: Vec<CssBlockChild>,
}

impl_css_serialize!(CssBlock, "Block", [children]);

#[derive(Debug, Clone)]
pub struct CssDeclaration {
    pub span: Span,
    pub property: String,
    pub value: String,
}

impl_css_serialize!(CssDeclaration, "Declaration", [property, value]);

#[derive(Debug, Clone)]
pub struct SelectorList {
    pub span: Span,
    pub children: Vec<ComplexSelector>,
}

impl_css_serialize!(SelectorList, "SelectorList", [children]);

#[derive(Debug, Clone)]
pub struct ComplexSelector {
    pub span: Span,
    pub children: Vec<RelativeSelector>,
}

impl_css_serialize!(ComplexSelector, "ComplexSelector", [children]);

#[derive(Debug, Clone)]
pub struct RelativeSelector {
    pub span: Span,
    pub combinator: Option<CssCombinator>,
    pub selectors: Vec<SimpleSelector>,
}

impl_css_serialize!(RelativeSelector, "RelativeSelector", [combinator, selectors]);

#[derive(Debug, Clone)]
pub struct CssCombinator {
    pub span: Span,
    pub name: String,
}

impl_css_serialize!(CssCombinator, "Combinator", [name]);

#[derive(Debug, Clone)]
pub struct TypeSelector {
    pub span: Span,
    pub name: String,
}

impl_css_serialize!(TypeSelector, "TypeSelector", [name]);

#[derive(Debug, Clone)]
pub struct IdSelector {
    pub span: Span,
    pub name: String,
}

impl_css_serialize!(IdSelector, "IdSelector", [name]);

#[derive(Debug, Clone)]
pub struct ClassSelector {
    pub span: Span,
    pub name: String,
}

impl_css_serialize!(ClassSelector, "ClassSelector", [name]);

#[derive(Debug, Clone)]
pub struct AttributeSelector {
    pub span: Span,
    pub name: String,
    pub matcher: Option<String>,
    pub value: Option<String>,
    pub flags: Option<String>,
}

impl_css_serialize!(AttributeSelector, "AttributeSelector", [name, matcher, value, flags]);

#[derive(Debug, Clone)]
pub struct PseudoClassSelector {
    pub span: Span,
    pub name: String,
    pub args: Option<Box<SelectorList>>,
}

impl_css_serialize!(PseudoClassSelector, "PseudoClassSelector", [name, args]);

#[derive(Debug, Clone)]
pub struct PseudoElementSelector {
    pub span: Span,
    pub name: String,
}

impl_css_serialize!(PseudoElementSelector, "PseudoElementSelector", [name]);

#[derive(Debug, Clone)]
pub struct NestingSelector {
    pub span: Span,
    pub name: String,
}

impl_css_serialize!(NestingSelector, "NestingSelector", [name]);

#[derive(Debug, Clone)]
pub struct Percentage {
    pub span: Span,
    pub value: String,
}

impl_css_serialize!(Percentage, "Percentage", [value]);

#[derive(Debug, Clone)]
pub struct Nth {
    pub span: Span,
    pub value: String,
}

impl_css_serialize!(Nth, "Nth", [value]);
