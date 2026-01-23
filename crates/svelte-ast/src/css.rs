use serde::Serialize;

use crate::node::AstNode;

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
    pub children: Vec<AstNode>,
    pub content: CssContent,
}

#[derive(Debug, Clone, Serialize)]
pub struct CssContent {
    pub start: u32,
    pub end: u32,
    pub styles: String,
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
    pub prelude: Box<AstNode>,
    pub block: Box<AstNode>,
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
    pub name: String,
    pub prelude: String,
    pub block: Option<Box<AstNode>>,
}

/*
 * interface Block extends BaseNode {
 *   type: 'Block';
 *   children: Array<Declaration | Rule | Atrule>;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct CssBlock {
    pub children: Vec<AstNode>,
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
    pub children: Vec<AstNode>,
}

/*
 * interface ComplexSelector extends BaseNode {
 *   type: 'ComplexSelector';
 *   children: RelativeSelector[];
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct ComplexSelector {
    pub children: Vec<AstNode>,
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
    pub combinator: Option<Box<AstNode>>,
    pub selectors: Vec<AstNode>,
}

/*
 * interface Combinator extends BaseNode {
 *   type: 'Combinator';
 *   name: string;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct CssCombinator {
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
    pub name: String,
    pub args: Option<Box<AstNode>>,
}

/*
 * interface PseudoElementSelector extends BaseNode {
 *   type: 'PseudoElementSelector';
 *   name: string;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct PseudoElementSelector {
    pub name: String,
}

/*
 * interface NestingSelector extends BaseNode {
 *   type: 'NestingSelector';
 *   name: '&';
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct NestingSelector;

/*
 * interface Percentage extends BaseNode {
 *   type: 'Percentage';
 *   value: string;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct Percentage {
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
    pub value: String,
}
