use serde::Serialize;

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
#[derive(Debug, Clone, Serialize)]
pub struct StyleSheet {
    #[serde(flatten)]
    pub span: Span,
    pub attributes: Vec<Attribute>,
    pub children: Vec<StyleSheetChild>,
    pub content: CssContent,
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
#[derive(Debug, Clone, Serialize)]
pub struct CssBlock {
    #[serde(flatten)]
    pub span: Span,
    pub children: Vec<CssBlockChild>,
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
#[derive(Debug, Clone, Serialize)]
pub struct SelectorList {
    #[serde(flatten)]
    pub span: Span,
    pub children: Vec<ComplexSelector>,
}

/*
 * interface ComplexSelector extends BaseNode {
 *   type: 'ComplexSelector';
 *   children: RelativeSelector[];
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct ComplexSelector {
    #[serde(flatten)]
    pub span: Span,
    pub children: Vec<RelativeSelector>,
}

/*
 * interface RelativeSelector extends BaseNode {
 *   type: 'RelativeSelector';
 *   combinator: Combinator | null;
 *   selectors: SimpleSelector[];
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct RelativeSelector {
    #[serde(flatten)]
    pub span: Span,
    pub combinator: Option<CssCombinator>,
    pub selectors: Vec<SimpleSelector>,
}

/*
 * interface Combinator extends BaseNode {
 *   type: 'Combinator';
 *   name: string;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct CssCombinator {
    #[serde(flatten)]
    pub span: Span,
    pub name: String,
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
