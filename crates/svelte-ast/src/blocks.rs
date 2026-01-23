use serde::Serialize;
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
    #[serde(serialize_with = "crate::utils::estree::serialize_opt_pat")]
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
#[derive(Debug, Clone, Serialize)]
pub struct SnippetBlock {
    #[serde(flatten)]
    pub span: Span,
    #[serde(serialize_with = "crate::utils::estree::serialize_boxed_ident")]
    pub expression: Box<swc::Ident>,
    #[serde(
        rename = "typeParams",
        skip_serializing_if = "Option::is_none"
    )]
    pub type_params: Option<String>,
    #[serde(serialize_with = "crate::utils::estree::serialize_pats")]
    pub parameters: Vec<swc::Pat>,
    pub body: Fragment,
}
