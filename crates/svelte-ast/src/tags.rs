use serde::Serialize;
use swc_ecma_ast as swc;

use crate::span::Span;

/*
 * interface ExpressionTag extends BaseNode {
 *   type: 'ExpressionTag';
 *   expression: Expression;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct ExpressionTag {
    pub span: Span,
    pub expression: Box<swc::Expr>,
}

/*
 * interface HtmlTag extends BaseNode {
 *   type: 'HtmlTag';
 *   expression: Expression;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct HtmlTag {
    pub span: Span,
    pub expression: Box<swc::Expr>,
}

/*
 * interface ConstTag extends BaseNode {
 *   type: 'ConstTag';
 *   declaration: VariableDeclaration;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct ConstTag {
    pub span: Span,
    pub declaration: Box<swc::VarDecl>,
}

/*
 * interface DebugTag extends BaseNode {
 *   type: 'DebugTag';
 *   identifiers: Identifier[];
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct DebugTag {
    pub span: Span,
    pub identifiers: Vec<swc::Ident>,
}

/*
 * interface RenderTag extends BaseNode {
 *   type: 'RenderTag';
 *   expression: SimpleCallExpression | (ChainExpression & { expression: SimpleCallExpression });
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct RenderTag {
    pub span: Span,
    pub expression: Box<swc::Expr>,
}

/*
 * interface AttachTag extends BaseNode {
 *   type: 'AttachTag';
 *   expression: Expression;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct AttachTag {
    pub span: Span,
    pub expression: Box<swc::Expr>,
}
