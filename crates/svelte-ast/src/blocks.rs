use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use swc_ecma_ast as swc;

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
    #[serde(serialize_with = "crate::utils::estree::serialize_boxed_expr")]
    pub test: Box<swc::Expr>,
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
    #[serde(serialize_with = "crate::utils::estree::serialize_boxed_expr")]
    pub expression: Box<swc::Expr>,
    #[serde(serialize_with = "crate::utils::estree::serialize_opt_pat_adjusted")]
    pub context: Option<Box<swc::Pat>>,
    pub body: Fragment,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback: Option<Fragment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "crate::utils::estree::serialize_opt_expr"
    )]
    pub key: Option<Box<swc::Expr>>,
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
    #[serde(serialize_with = "crate::utils::estree::serialize_boxed_expr")]
    pub expression: Box<swc::Expr>,
    #[serde(serialize_with = "crate::utils::estree::serialize_opt_pat")]
    pub value: Option<Box<swc::Pat>>,
    #[serde(serialize_with = "crate::utils::estree::serialize_opt_pat")]
    pub error: Option<Box<swc::Pat>>,
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
    #[serde(serialize_with = "crate::utils::estree::serialize_boxed_expr")]
    pub expression: Box<swc::Expr>,
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
    pub expression: Box<swc::Ident>,
    pub type_params: Option<String>,
    pub parameters: Vec<swc::Pat>,
    pub body: Fragment,
}

impl Serialize for SnippetBlock {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("type", "SnippetBlock")?;

        // Expression: Identifier with character-inclusive loc (matches reference Svelte)
        map.serialize_entry(
            "expression",
            &crate::utils::estree::IdentWithCharLoc(&self.expression),
        )?;

        if let Some(ref tp) = self.type_params {
            map.serialize_entry("typeParams", tp)?;
        }

        // Parameters go through normal ESTree transform
        let params_val = serde_json::to_value(&self.parameters)
            .map_err(serde::ser::Error::custom)?;
        let params_transformed = crate::utils::estree::transform_value_pub(params_val);
        map.serialize_entry("parameters", &params_transformed)?;

        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}
