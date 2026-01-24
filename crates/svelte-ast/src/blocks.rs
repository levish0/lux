use oxc_ast::ast::{BindingPattern, Expression};

use crate::metadata::{ExpressionNodeMetadata, SnippetBlockMetadata};
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
#[derive(Debug)]
pub struct IfBlock<'a> {
    pub span: Span,
    pub elseif: bool,
    pub test: Expression<'a>,
    pub consequent: Fragment<'a>,
    pub alternate: Option<Fragment<'a>>,
    pub metadata: ExpressionNodeMetadata,
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
    pub index: Option<String>,
    pub key: Option<Expression<'a>>,
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
    pub parameters: Vec<BindingPattern<'a>>,
    pub type_params: Option<String>,
    pub body: Fragment<'a>,
    pub metadata: SnippetBlockMetadata,
}
