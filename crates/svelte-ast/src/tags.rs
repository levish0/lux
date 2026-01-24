use oxc_ast::ast::{Expression, VariableDeclaration};

use crate::span::Span;

/*
 * interface ExpressionTag extends BaseNode {
 *   type: 'ExpressionTag';
 *   expression: Expression;
 * }
 */
#[derive(Debug)]
pub struct ExpressionTag<'a> {
    pub span: Span,
    pub expression: Expression<'a>,
}

/*
 * interface HtmlTag extends BaseNode {
 *   type: 'HtmlTag';
 *   expression: Expression;
 * }
 */
#[derive(Debug)]
pub struct HtmlTag<'a> {
    pub span: Span,
    pub expression: Expression<'a>,
}

/*
 * interface ConstTag extends BaseNode {
 *   type: 'ConstTag';
 *   declaration: VariableDeclaration;
 * }
 */
#[derive(Debug)]
pub struct ConstTag<'a> {
    pub span: Span,
    pub declaration: VariableDeclaration<'a>,
}

/*
 * interface DebugTag extends BaseNode {
 *   type: 'DebugTag';
 *   identifiers: Identifier[];
 * }
 */
#[derive(Debug)]
pub struct DebugTag<'a> {
    pub span: Span,
    pub identifiers: Vec<Expression<'a>>,
}

/*
 * interface RenderTag extends BaseNode {
 *   type: 'RenderTag';
 *   expression: SimpleCallExpression | (ChainExpression & { expression: SimpleCallExpression });
 * }
 */
#[derive(Debug)]
pub struct RenderTag<'a> {
    pub span: Span,
    pub expression: Expression<'a>,
}

/*
 * interface AttachTag extends BaseNode {
 *   type: 'AttachTag';
 *   expression: Expression;
 * }
 */
#[derive(Debug)]
pub struct AttachTag<'a> {
    pub span: Span,
    pub expression: Expression<'a>,
}
