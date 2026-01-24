use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};

use crate::JsNode;
use crate::root::Fragment;
use crate::span::Span;

/*
 * interface IfBlock extends BaseNode {
 *   type: 'IfBlock';
 *   elseif: boolean;
 *   test: Expression;
 *   consequent: Fragment;
 *   alternate: Fragment | null;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct IfBlock {
    #[serde(flatten)]
    pub span: Span,
    pub elseif: bool,
    pub test: JsNode,
    pub consequent: Fragment,
    pub alternate: Option<Fragment>,
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
#[derive(Debug, Clone, Serialize)]
pub struct EachBlock {
    #[serde(flatten)]
    pub span: Span,
    pub expression: JsNode,
    pub context: Option<JsNode>,
    pub body: Fragment,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback: Option<Fragment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<JsNode>,
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
#[derive(Debug, Clone, Serialize)]
pub struct AwaitBlock {
    #[serde(flatten)]
    pub span: Span,
    pub expression: JsNode,
    pub value: Option<JsNode>,
    pub error: Option<JsNode>,
    pub pending: Option<Fragment>,
    pub then: Option<Fragment>,
    pub catch: Option<Fragment>,
}

/*
 * interface KeyBlock extends BaseNode {
 *   type: 'KeyBlock';
 *   expression: Expression;
 *   fragment: Fragment;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct KeyBlock {
    #[serde(flatten)]
    pub span: Span,
    pub expression: JsNode,
    pub fragment: Fragment,
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
#[derive(Debug, Clone)]
pub struct SnippetBlock {
    pub span: Span,
    pub expression: JsNode,
    pub type_params: Option<String>,
    pub parameters: JsNode,
    pub body: Fragment,
}

impl Serialize for SnippetBlock {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("type", "SnippetBlock")?;
        map.serialize_entry("expression", &self.expression)?;

        if let Some(ref tp) = self.type_params {
            map.serialize_entry("typeParams", tp)?;
        }

        map.serialize_entry("parameters", &self.parameters)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}
