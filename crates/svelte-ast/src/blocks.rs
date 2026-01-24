use oxc_ast::ast::{BindingPattern, Expression, FormalParameter};
use serde::Serialize;
use serde::ser::SerializeMap;

use crate::metadata::{ExpressionNodeMetadata, SnippetBlockMetadata};
use crate::root::Fragment;
use crate::span::Span;
use crate::utils::estree::{OxcOptionSerialize, OxcSerialize, OxcVecSerialize};

/*
 * interface IfBlock extends BaseNode {
 *   type: 'IfBlock';
 *   elseif: boolean;
 *   test: Expression;
 *   consequent: Fragment;
 *   alternate: Fragment | null;
 * }
 */
#[derive(Debug)]
pub struct IfBlock<'a> {
    pub span: Span,
    pub elseif: bool,
    pub test: Expression<'a>,
    pub consequent: Fragment<'a>,
    pub alternate: Option<Fragment<'a>>,
    pub metadata: ExpressionNodeMetadata,
}

impl Serialize for IfBlock<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "IfBlock")?;
        map.serialize_entry("elseif", &self.elseif)?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("test", &OxcSerialize(&self.test))?;
        map.serialize_entry("consequent", &self.consequent)?;
        map.serialize_entry("alternate", &self.alternate)?;
        map.end()
    }
}

/*
 * interface EachBlock extends BaseNode {
 *   type: 'EachBlock';
 *   expression: Expression;
 *   context: Pattern | null;
 *   body: Fragment;
 *   fallback?: Fragment;
 *   index?: string;
 *   key?: Expression;
 * }
 */
#[derive(Debug)]
pub struct EachBlock<'a> {
    pub span: Span,
    pub expression: Expression<'a>,
    pub context: Option<BindingPattern<'a>>,
    pub body: Fragment<'a>,
    pub fallback: Option<Fragment<'a>>,
    pub index: Option<&'a str>,
    pub key: Option<Expression<'a>>,
}

impl Serialize for EachBlock<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "EachBlock")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("expression", &OxcSerialize(&self.expression))?;
        map.serialize_entry("context", &OxcOptionSerialize(&self.context))?;
        map.serialize_entry("body", &self.body)?;
        map.serialize_entry("fallback", &self.fallback)?;
        map.serialize_entry("index", &self.index)?;
        map.serialize_entry("key", &OxcOptionSerialize(&self.key))?;
        map.end()
    }
}

/*
 * interface AwaitBlock extends BaseNode {
 *   type: 'AwaitBlock';
 *   expression: Expression;
 *   value: Pattern | null;
 *   error: Pattern | null;
 *   pending: Fragment | null;
 *   then: Fragment | null;
 *   catch: Fragment | null;
 * }
 */
#[derive(Debug)]
pub struct AwaitBlock<'a> {
    pub span: Span,
    pub expression: Expression<'a>,
    pub value: Option<BindingPattern<'a>>,
    pub error: Option<BindingPattern<'a>>,
    pub pending: Option<Fragment<'a>>,
    pub then: Option<Fragment<'a>>,
    pub catch: Option<Fragment<'a>>,
    pub metadata: ExpressionNodeMetadata,
}

impl Serialize for AwaitBlock<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "AwaitBlock")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("expression", &OxcSerialize(&self.expression))?;
        map.serialize_entry("value", &OxcOptionSerialize(&self.value))?;
        map.serialize_entry("error", &OxcOptionSerialize(&self.error))?;
        map.serialize_entry("pending", &self.pending)?;
        map.serialize_entry("then", &self.then)?;
        map.serialize_entry("catch", &self.catch)?;
        map.end()
    }
}

/*
 * interface KeyBlock extends BaseNode {
 *   type: 'KeyBlock';
 *   expression: Expression;
 *   fragment: Fragment;
 * }
 */
#[derive(Debug)]
pub struct KeyBlock<'a> {
    pub span: Span,
    pub expression: Expression<'a>,
    pub fragment: Fragment<'a>,
    pub metadata: ExpressionNodeMetadata,
}

impl Serialize for KeyBlock<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "KeyBlock")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("expression", &OxcSerialize(&self.expression))?;
        map.serialize_entry("fragment", &self.fragment)?;
        map.end()
    }
}

/*
 * interface SnippetBlock extends BaseNode {
 *   type: 'SnippetBlock';
 *   expression: Identifier;
 *   parameters: Pattern[];
 *   typeParams?: string;
 *   body: Fragment;
 * }
 */
#[derive(Debug)]
pub struct SnippetBlock<'a> {
    pub span: Span,
    pub expression: Expression<'a>,
    pub parameters: Vec<FormalParameter<'a>>,
    pub type_params: Option<&'a str>,
    pub body: Fragment<'a>,
    pub metadata: SnippetBlockMetadata,
}

impl Serialize for SnippetBlock<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "SnippetBlock")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("expression", &OxcSerialize(&self.expression))?;
        map.serialize_entry("parameters", &OxcVecSerialize(&self.parameters))?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}
