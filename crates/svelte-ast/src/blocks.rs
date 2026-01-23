use serde::Serialize;
use swc_ecma_ast as swc;

use crate::root::Fragment;

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
    pub elseif: bool,
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
    pub expression: Box<swc::Expr>,
    pub context: Option<Box<swc::Pat>>,
    pub body: Fragment,
    pub fallback: Option<Fragment>,
    pub index: Option<String>,
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
    pub expression: Box<swc::Expr>,
    pub value: Option<Box<swc::Pat>>,
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
    pub expression: Box<swc::Expr>,
    pub fragment: Fragment,
}

/*
 * interface SnippetBlock extends BaseNode {
 *   type: 'SnippetBlock';
 *   expression: Identifier;
 *   parameters: Pattern[];
 *   body: Fragment;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct SnippetBlock {
    pub expression: Box<swc::Ident>,
    pub parameters: Vec<swc::Pat>,
    pub body: Fragment,
}
