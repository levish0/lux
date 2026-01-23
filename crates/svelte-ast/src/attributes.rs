use serde::Serialize;
use swc_ecma_ast as swc;

use crate::span::Span;
use crate::tags::ExpressionTag;
use crate::text::Text;

/*
 * interface Attribute extends BaseNode {
 *   type: 'Attribute';
 *   name: string;
 *   value: true | ExpressionTag | Array<Text | ExpressionTag>;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct Attribute {
    pub span: Span,
    pub name: String,
    pub value: AttributeValue,
}

#[derive(Debug, Clone, Serialize)]
pub enum AttributeValue {
    True,
    Expression(ExpressionTag),
    Sequence(Vec<AttributeSequenceValue>),
}

#[derive(Debug, Clone, Serialize)]
pub enum AttributeSequenceValue {
    Text(Text),
    Expression(ExpressionTag),
}

/*
 * interface SpreadAttribute extends BaseNode {
 *   type: 'SpreadAttribute';
 *   expression: Expression;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct SpreadAttribute {
    pub span: Span,
    pub expression: Box<swc::Expr>,
}

/*
 * interface BindDirective extends BaseNode {
 *   type: 'BindDirective';
 *   name: string;
 *   expression: Identifier | MemberExpression | SequenceExpression;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct BindDirective {
    pub span: Span,
    pub name: String,
    pub expression: Box<swc::Expr>,
}

/*
 * interface ClassDirective extends BaseNode {
 *   type: 'ClassDirective';
 *   name: 'class';
 *   expression: Expression;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct ClassDirective {
    pub span: Span,
    pub name: String,
    pub expression: Box<swc::Expr>,
}

/*
 * interface StyleDirective extends BaseNode {
 *   type: 'StyleDirective';
 *   name: string;
 *   value: true | ExpressionTag | Array<ExpressionTag | Text>;
 *   modifiers: Array<'important'>;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct StyleDirective {
    pub span: Span,
    pub name: String,
    pub value: AttributeValue,
    pub modifiers: Vec<StyleModifier>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum StyleModifier {
    Important,
}

/*
 * interface OnDirective extends BaseNode {
 *   type: 'OnDirective';
 *   name: string;
 *   expression: null | Expression;
 *   modifiers: string[];
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct OnDirective {
    pub span: Span,
    pub name: String,
    pub expression: Option<Box<swc::Expr>>,
    pub modifiers: Vec<EventModifier>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum EventModifier {
    Capture,
    Nonpassive,
    Once,
    Passive,
    PreventDefault,
    Self_,
    StopImmediatePropagation,
    StopPropagation,
    Trusted,
}

/*
 * interface TransitionDirective extends BaseNode {
 *   type: 'TransitionDirective';
 *   name: string;
 *   expression: null | Expression;
 *   modifiers: Array<'local' | 'global'>;
 *   intro: boolean;
 *   outro: boolean;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct TransitionDirective {
    pub span: Span,
    pub name: String,
    pub expression: Option<Box<swc::Expr>>,
    pub modifiers: Vec<TransitionModifier>,
    pub intro: bool,
    pub outro: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TransitionModifier {
    Local,
    Global,
}

/*
 * interface AnimateDirective extends BaseNode {
 *   type: 'AnimateDirective';
 *   name: string;
 *   expression: null | Expression;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct AnimateDirective {
    pub span: Span,
    pub name: String,
    pub expression: Option<Box<swc::Expr>>,
}

/*
 * interface UseDirective extends BaseNode {
 *   type: 'UseDirective';
 *   name: string;
 *   expression: null | Expression;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct UseDirective {
    pub span: Span,
    pub name: String,
    pub expression: Option<Box<swc::Expr>>,
}

/*
 * interface LetDirective extends BaseNode {
 *   type: 'LetDirective';
 *   name: string;
 *   expression: null | Identifier | ArrayExpression | ObjectExpression;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct LetDirective {
    pub span: Span,
    pub name: String,
    pub expression: Option<Box<swc::Expr>>,
}
