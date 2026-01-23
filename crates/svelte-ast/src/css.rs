use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};

use crate::attributes::Attribute;
use crate::node::{CssBlockChild, SimpleSelector, StyleSheetChild};
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
#[derive(Debug, Clone)]
pub struct StyleSheet {
    pub span: Span,
    pub attributes: Vec<Attribute>,
    pub children: Vec<StyleSheetChild>,
    pub content: CssContent,
}

impl Serialize for StyleSheet {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
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
pub struct CssContent {
    pub start: u32,
    pub end: u32,
    pub styles: String,
    pub comment: Option<Comment>,
}

/*
 * interface Rule extends BaseNode {
 *   type: 'Rule';
 *   prelude: SelectorList;
 *   block: Block;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct CssRule {
    #[serde(flatten)]
    pub span: Span,
    pub prelude: SelectorList,
    pub block: CssBlock,
}

/*
 * interface Atrule extends BaseNode {
 *   type: 'Atrule';
 *   name: string;
 *   prelude: string;
 *   block: Block | null;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct CssAtrule {
    #[serde(flatten)]
    pub span: Span,
    pub name: String,
    pub prelude: String,
    pub block: Option<CssBlock>,
}

/*
 * interface Block extends BaseNode {
 *   type: 'Block';
 *   children: Array<Declaration | Rule | Atrule>;
 * }
 */
#[derive(Debug, Clone)]
pub struct CssBlock {
    pub span: Span,
    pub children: Vec<CssBlockChild>,
}

impl Serialize for CssBlock {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "Block")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("children", &self.children)?;
        map.end()
    }
}

/*
 * interface Declaration extends BaseNode {
 *   type: 'Declaration';
 *   property: string;
 *   value: string;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct CssDeclaration {
    #[serde(flatten)]
    pub span: Span,
    pub property: String,
    pub value: String,
}

/*
 * interface SelectorList extends BaseNode {
 *   type: 'SelectorList';
 *   children: ComplexSelector[];
 * }
 */
#[derive(Debug, Clone)]
pub struct SelectorList {
    pub span: Span,
    pub children: Vec<ComplexSelector>,
}

impl Serialize for SelectorList {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "SelectorList")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("children", &self.children)?;
        map.end()
    }
}

/*
 * interface ComplexSelector extends BaseNode {
 *   type: 'ComplexSelector';
 *   children: RelativeSelector[];
 * }
 */
#[derive(Debug, Clone)]
pub struct ComplexSelector {
    pub span: Span,
    pub children: Vec<RelativeSelector>,
}

impl Serialize for ComplexSelector {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "ComplexSelector")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("children", &self.children)?;
        map.end()
    }
}

/*
 * interface RelativeSelector extends BaseNode {
 *   type: 'RelativeSelector';
 *   combinator: Combinator | null;
 *   selectors: SimpleSelector[];
 * }
 */
#[derive(Debug, Clone)]
pub struct RelativeSelector {
    pub span: Span,
    pub combinator: Option<CssCombinator>,
    pub selectors: Vec<SimpleSelector>,
}

impl Serialize for RelativeSelector {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "RelativeSelector")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("combinator", &self.combinator)?;
        map.serialize_entry("selectors", &self.selectors)?;
        map.end()
    }
}

/*
 * interface Combinator extends BaseNode {
 *   type: 'Combinator';
 *   name: string;
 * }
 */
#[derive(Debug, Clone)]
pub struct CssCombinator {
    pub span: Span,
    pub name: String,
}

impl Serialize for CssCombinator {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "Combinator")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("name", &self.name)?;
        map.end()
    }
}

/*
 * interface TypeSelector extends BaseNode {
 *   type: 'TypeSelector';
 *   name: string;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct TypeSelector {
    #[serde(flatten)]
    pub span: Span,
    pub name: String,
}

/*
 * interface IdSelector extends BaseNode {
 *   type: 'IdSelector';
 *   name: string;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct IdSelector {
    #[serde(flatten)]
    pub span: Span,
    pub name: String,
}

/*
 * interface ClassSelector extends BaseNode {
 *   type: 'ClassSelector';
 *   name: string;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct ClassSelector {
    #[serde(flatten)]
    pub span: Span,
    pub name: String,
}

/*
 * interface AttributeSelector extends BaseNode {
 *   type: 'AttributeSelector';
 *   name: string;
 *   matcher: string | null;
 *   value: string | null;
 *   flags: string | null;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct AttributeSelector {
    #[serde(flatten)]
    pub span: Span,
    pub name: String,
    pub matcher: Option<String>,
    pub value: Option<String>,
    pub flags: Option<String>,
}

/*
 * interface PseudoClassSelector extends BaseNode {
 *   type: 'PseudoClassSelector';
 *   name: string;
 *   args: SelectorList | null;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct PseudoClassSelector {
    #[serde(flatten)]
    pub span: Span,
    pub name: String,
    pub args: Option<Box<SelectorList>>,
}

/*
 * interface PseudoElementSelector extends BaseNode {
 *   type: 'PseudoElementSelector';
 *   name: string;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct PseudoElementSelector {
    #[serde(flatten)]
    pub span: Span,
    pub name: String,
}

/*
 * interface NestingSelector extends BaseNode {
 *   type: 'NestingSelector';
 *   name: '&';
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct NestingSelector {
    #[serde(flatten)]
    pub span: Span,
    pub name: String,
}

/*
 * interface Percentage extends BaseNode {
 *   type: 'Percentage';
 *   value: string;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct Percentage {
    #[serde(flatten)]
    pub span: Span,
    pub value: String,
}

/*
 * interface Nth extends BaseNode {
 *   type: 'Nth';
 *   value: string;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct Nth {
    #[serde(flatten)]
    pub span: Span,
    pub value: String,
}
