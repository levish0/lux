use serde::Serialize;
use swc_ecma_ast as swc;

use crate::span::Span;
use crate::tags::ExpressionTag;
use crate::text::Text;

/*
 * interface Attribute extends BaseNode {
 *   type: 'Attribute';
 *   name: string;
 *   name_loc: SourceLocation | null;
 *   value: true | ExpressionTag | Array<Text | ExpressionTag>;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct Attribute {
    #[serde(flatten)]
    pub span: Span,
    pub name: String,
    pub name_loc: Option<Span>,
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
 *   name_loc: SourceLocation | null;
 *   expression: Expression;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct SpreadAttribute {
    #[serde(flatten)]
    pub span: Span,
    pub name_loc: Option<Span>,
    pub expression: Box<swc::Expr>,
}

/*
 * interface BindDirective extends BaseNode {
 *   type: 'BindDirective';
 *   name: string;
 *   name_loc: SourceLocation | null;
 *   expression: Identifier | MemberExpression | SequenceExpression;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct BindDirective {
    #[serde(flatten)]
    pub span: Span,
    pub name: String,
    pub name_loc: Option<Span>,
    pub expression: Box<swc::Expr>,
}

/*
 * interface ClassDirective extends BaseNode {
 *   type: 'ClassDirective';
 *   name: 'class';
 *   name_loc: SourceLocation | null;
 *   expression: Expression;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct ClassDirective {
    #[serde(flatten)]
    pub span: Span,
    pub name: String,
    pub name_loc: Option<Span>,
    pub expression: Box<swc::Expr>,
}

/*
 * interface StyleDirective extends BaseNode {
 *   type: 'StyleDirective';
 *   name: string;
 *   name_loc: SourceLocation | null;
 *   value: true | ExpressionTag | Array<ExpressionTag | Text>;
 *   modifiers: Array<'important'>;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct StyleDirective {
    #[serde(flatten)]
    pub span: Span,
    pub name: String,
    pub name_loc: Option<Span>,
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
 *   name_loc: SourceLocation | null;
 *   expression: null | Expression;
 *   modifiers: string[];
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct OnDirective {
    #[serde(flatten)]
    pub span: Span,
    pub name: String,
    pub name_loc: Option<Span>,
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
 *   name_loc: SourceLocation | null;
 *   expression: null | Expression;
 *   modifiers: Array<'local' | 'global'>;
 *   intro: boolean;
 *   outro: boolean;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct TransitionDirective {
    #[serde(flatten)]
    pub span: Span,
    pub name: String,
    pub name_loc: Option<Span>,
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
 *   name_loc: SourceLocation | null;
 *   expression: null | Expression;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct AnimateDirective {
    #[serde(flatten)]
    pub span: Span,
    pub name: String,
    pub name_loc: Option<Span>,
    pub expression: Option<Box<swc::Expr>>,
}

/*
 * interface UseDirective extends BaseNode {
 *   type: 'UseDirective';
 *   name: string;
 *   name_loc: SourceLocation | null;
 *   expression: null | Expression;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct UseDirective {
    #[serde(flatten)]
    pub span: Span,
    pub name: String,
    pub name_loc: Option<Span>,
    pub expression: Option<Box<swc::Expr>>,
}

/*
 * interface LetDirective extends BaseNode {
 *   type: 'LetDirective';
 *   name: string;
 *   name_loc: SourceLocation | null;
 *   expression: null | Identifier | ArrayExpression | ObjectExpression;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct LetDirective {
    #[serde(flatten)]
    pub span: Span,
    pub name: String,
    pub name_loc: Option<Span>,
    pub expression: Option<Box<swc::Expr>>,
}
